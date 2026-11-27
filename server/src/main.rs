use axum::{Router, extract::State, http::StatusCode, routing::get};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use argon2::{
    Argon2, password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng}
};
use axum::{Json, routing::{post, put}};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};
use axum::extract::Path;
use axum::http::{HeaderMap, header::AUTHORIZATION};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::time::{Duration, Instant};
use sqlx::PgPool;

// 登録のリクエスト
#[derive(Deserialize)]
struct RegisterRequest {
    user_id: String,
    display_name: String,
    password: String,
    endpoint_id: String,
}

// アプリの状態管理
#[derive(Clone)]
struct AppState {
    pool: PgPool,
    online: Arc<Mutex<HashMap<String, Instant>>>,
    jwt_secret: String,
    read_only: bool,
}

// ログインリクエスト
#[derive(Deserialize)]
struct LoginRequest {
    user_id: String,
    password: String,
}

// ログイン成功時のレスポンス(JWT)
#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

// JWTの中身。sub=ユーザーID, exp=有効期限(UNIX秒)
#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

// ユーザー情報取得(GET /users/{user_id})のレスポンス
#[derive(Serialize)]
struct UserResponse {
    user_id: String,
    display_name: String,
    endpoint_id: String,
    online: bool,
}

// 最後のheartbeatから60秒以内ならオンラインとみなす
const ONLINE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Deserialize)]
struct UpdateEndpointRequest {
    endpoint_id: String,
}

// メイン
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    // LYNQ_JWT_SECRETが未設定ならpanicして起動を中止する 
    let jwt_secret = std::env::var("LYNQ_JWT_SECRET").expect("LYNQ_JWT_SECRETが未設定");

    let read_only = std::env::var("LYNQ_READ_ONLY").is_ok_and(|v| v == "true");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URLが未設定");
    let pool = PgPool::connect(&database_url).await.expect("DBに接続できない");

    if !read_only {
        sqlx::migrate!().run(&pool).await.expect("マイグレーション失敗");
    }

    // 全ハンドラで共有する状態を作る
    // Arc: 複数のリクエスト(スレッド)から同じデータを参照するため
    // Mutex: 同時に書き換えられないようにロックするため
    let state = AppState {
        pool,
        online: Arc::new(Mutex::new(HashMap::new())),
        read_only,
        jwt_secret,
    };

    
    let app = Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/users/{user_id}", get(get_user))
        .route("/me/heartbeat", post(heartbeat))
        .route("/me/endpoint_id", put(endpoint_responcse))
        .with_state(state);

    // 全てのネットワークインターフェースのポート8080で待ち受ける
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn register (
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> StatusCode {
    if state.read_only {
        return StatusCode::FORBIDDEN;
    }

    if req.user_id.is_empty() || req.password.len() < 8 {
        return StatusCode::BAD_REQUEST;
    }
    
    // ソルトを作成する
    let salt = SaltString::generate(&mut OsRng);
    
    // ハッシュ化したpasswordにはソルトを付与する
    let password_hash = match Argon2::default().hash_password(req.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let result = sqlx::query(
        "INSERT INTO users (user_id, display_name, password_hash, endpoint_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(&req.user_id)
    .bind(&req.display_name)
    .bind(&password_hash)
    .bind(&req.endpoint_id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => StatusCode::CREATED,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => StatusCode::CONFLICT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// ログイン: パスワードが合っていればJWTを発行して返す
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // 保存済みのパスワードハッシュを取り出す
    // ブロック内でロックを取り、値をcloneしたらすぐにロックを解放する
    // (重いArgon2の検証中に他のリクエストを待たせないため)
    let password_hash = sqlx::query_scalar::<_, String>(
        "SELECT password_hash FROM users WHERE user_id = $1",
    )
    .bind(&req.user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    // ユーザーが存在しない場合も401を返す
    // (存在するユーザーIDかどうかを外部に知られないようにするため)
    .ok_or(StatusCode::UNAUTHORIZED)?;

    // ハッシュ文字列を解析する(ソルトやパラメータもこの中に含まれている)
    let parsed = PasswordHash::new(&password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // 入力されたパスワードが保存済みハッシュと一致するか検証する
    if Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed)
        .is_err()
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // JWTの中身を作る。有効期限は現在時刻から7日後(UNIX秒)
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = Claims {
        sub: req.user_id,
        exp: now + 60 * 60 * 24 * 7,
    };

    // サーバーの秘密鍵で署名してJWTを作る(アルゴリズムはデフォルトのHS256)
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse { token }))
}

fn verify_token(headers: &HeaderMap, secret: &str) -> Result<String, StatusCode> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = value
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(data.claims.sub)
}

async fn heartbeat(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    let user_id = match verify_token(&headers, &state.jwt_secret) {
        Ok(id) => id,
        Err(code) => return code,
    };

    state.online.lock().unwrap().insert(user_id, Instant::now());
    StatusCode::NO_CONTENT
}

async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> Result<Json<UserResponse>, StatusCode> {
    verify_token(&headers, &state.jwt_secret)?;

    let (display_name, endpoint_id) = sqlx::query_as::<_, (String, String)>(
        "SELECT display_name, endpoint_id FROM users WHERE user_id = $1",
    )
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let online = {
        let online = state.online.lock().unwrap();
        match online.get(&user_id) {
            Some(last) => last.elapsed() < ONLINE_TIMEOUT,
            None => false,
        }
    };

    Ok(Json(UserResponse {
        user_id,
        display_name,
        endpoint_id,
        online,
    }))
}

async fn endpoint_responcse(

    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateEndpointRequest>,
) -> StatusCode {
    // registerの先頭と同じ読み取り専用チェック
    if state.read_only {
        return StatusCode::FORBIDDEN;
    }

    // heartbeatの先頭と同じ、verify_tokenでuser_idを取り出すmatch
    let user_id = match verify_token(&headers, &state.jwt_secret) {
        Ok(id) => id,
        Err(code) => return code,
    };

    let result = sqlx::query("UPDATE users SET endpoint_id = $1 WHERE user_id = $2")
        // .bindを2つ（$1と$2の順）
        .bind(&req.endpoint_id)
        .bind(&user_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => StatusCode::NOT_FOUND,
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}