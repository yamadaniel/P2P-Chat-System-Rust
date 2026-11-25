use axum::{Router, extract::State, http::StatusCode, routing::get};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use argon2::{
    Argon2, password_hash::{PasswordHasher, SaltString, rand_core::OsRng, PasswordVerifier}
};
use axum::{Json, routing::post};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::{SystemTime. UNIX_EPOCH};

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
    jwt_secret: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    user_id: String,
    password: String,
}

#[derive(Serialize)]
struct LoginRequponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("LYNQ_JWT_SECRET").expect("LYNQ_JWT_SECRETが未設定");



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