use axum::{extract::State, http::StatusCode, response::Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tracing::error;
use validator::Validate;

use crate::models::{CreateUserRequest, LoginRequest, LoginResponse, UserRole};
use crate::services::user_service;

#[derive(serde::Serialize, serde::Deserialize)]
struct Claims {
    sub: String, // Subject (user ID)
    exp: usize,  // Expiration time
    iat: usize,  // Issued at
}

pub async fn register(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("User registration validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Hash password
    let password_hash = match hash(&payload.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(err) => {
            error!("Failed to hash password: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let role = payload.role.unwrap_or(UserRole::User);

    match user_service::create_user(
        &db,
        payload.email,
        payload.username,
        password_hash,
        payload.full_name,
        role,
    )
    .await
    {
        Ok(user) => Ok(Json(json!({
            "message": "User registered successfully",
            "user": {
                "id": user.id,
                "email": user.email,
                "username": user.username,
                "full_name": user.full_name,
                "role": user.role
            }
        }))),
        Err(err) => {
            error!("Failed to create user: {:?}", err);
            if err.to_string().contains("UNIQUE constraint failed") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn login(
    State(db): State<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("Login validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Find user by email
    let user = match user_service::get_user_by_email(&db, &payload.email).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(err) => {
            error!("Failed to get user: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Verify password
    if !verify(&payload.password, &user.password_hash).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Check if user is active
    if !user.is_active {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate JWT token
    let claims = Claims {
        sub: user.id.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("your-256-bit-secret-key-here-change-in-production".as_ref()),
    ) {
        Ok(token) => token,
        Err(err) => {
            error!("Failed to generate JWT token: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(LoginResponse {
        token,
        user: user.into(),
    }))
}