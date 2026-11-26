use axum::{Router, extract::State, http::StatusCode, routing::get};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use argon2::{
    Argon2, password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng}
};
use axum::{Json, routing::post};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};
use axum::extract::Path;
use axum::http::{HeaderMap, header::AUTHORIZATION};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::time::{Duration, Instant};

#[derive(Deserialize)]
struct RegisterRequest {
    user_id: String,
    display_name: String,
    password: String,
    endpoint_id: String,
}

struct User {
    display_name: String,
    password_hash: String,
    endpoint_id: String,
}

#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<HashMap<String, User>>>,
    online: Arc<Mutex<HashMap<String, Instant>>>,
    jwt_secret: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    user_id: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Serialize)]
struct UserResponse {
    user_id: String,
    display_name: String,
    endpoint_id: String,
    online: bool,
}

const ONLINE_TIMEOUT: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("LYNQ_JWT_SECRET").expect("LYNQ_JWT_SECRETが未設定");

    let state = AppState {
        users: Arc::new(Mutex::new(HashMap::new())),
        online: Arc::new(Mutex::new(HashMap::new())),
        jwt_secret,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/users/{user_id}", get(get_user))
        .route("/me/heartneat", post(heartbeat))
        .with_state(state);

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
    if req.user_id.is_empty() || req.password.len() < 8 {
        return StatusCode::BAD_REQUEST;
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = match Argon2::default().hash_password(req.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let mut users = state.users.lock().unwrap();
    if users.contains_key(&req.user_id) {
        return StatusCode::CONFLICT;
    }
    users.insert(
        req.user_id,
        User {
            display_name: req.display_name,
            password_hash,
            endpoint_id: req.endpoint_id,
        },
    );

    StatusCode::CREATED
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let password_hash = {
        let users = state.users.lock().unwrap();
        match users.get(&req.user_id) {
            Some(user) => user.password_hash.clone(),
            None => return Err(StatusCode::UNAUTHORIZED),
        }
    };

    let parsed = PasswordHash::new(&password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed)
        .is_err()
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = Claims {
        sub: req.user_id,
        exp: now + 60 * 60 * 24 * 7,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse { token }))
}

