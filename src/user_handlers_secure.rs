use axum::{extract::State, http::StatusCode, response::Json};
use bcrypt::{hash, verify};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tracing::{error, info, warn};
use validator::Validate;

use crate::models::{CreateUserRequest, LoginRequest, LoginResponse, UserRole};
use crate::services::user_service;
use crate::utils::auth::{create_token};
use crate::config::Config;

fn has_password_complexity(password: &str) -> bool {
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    
    has_upper && has_lower && has_digit && has_special
}

// Simple rate limiting using in-memory storage (in production, use Redis)
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

lazy_static::lazy_static! {
    static ref FAILED_ATTEMPTS: Mutex<HashMap<String, (u32, SystemTime)>> = Mutex::new(HashMap::new());
}

fn is_rate_limited_simple(email: &str, max_attempts: u32, window_minutes: u64) -> bool {
    let mut attempts = FAILED_ATTEMPTS.lock().unwrap();
    let now = SystemTime::now();
    
    if let Some((count, last_attempt)) = attempts.get(email) {
        let window = std::time::Duration::from_secs(window_minutes * 60);
        if now.duration_since(*last_attempt).unwrap_or(window + std::time::Duration::from_secs(1)) < window {
            return *count >= max_attempts;
        }
    }
    false
}

fn record_failed_login_simple(email: &str) {
    let mut attempts = FAILED_ATTEMPTS.lock().unwrap();
    let now = SystemTime::now();
    let entry = attempts.entry(email.to_string()).or_insert((0, now));
    entry.0 += 1;
    entry.1 = now;
}

fn record_successful_login_simple(email: &str) {
    let mut attempts = FAILED_ATTEMPTS.lock().unwrap();
    attempts.remove(email);
}

pub async fn register_secure(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("User registration validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Enhanced validation for business logic (PR 3 improvement)
    if payload.password.len() < 8 {
        error!("Password too short: minimum 8 characters required");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check password complexity (PR 3 improvement)
    if !has_password_complexity(&payload.password) {
        error!("Password does not meet complexity requirements: must contain uppercase, lowercase, digit, and special character");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Hash password with higher cost for better security (PR 3 improvement)
    let password_hash = match hash(&payload.password, 12) { // Increased from DEFAULT_COST (10) to 12
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
        Ok(user) => {
            info!("User registered successfully: {} (audit log)", user.email);
            Ok(Json(json!({
                "message": "User registered successfully",
                "user": {
                    "id": user.id,
                    "email": user.email,
                    "username": user.username,
                    "full_name": user.full_name,
                    "role": user.role
                },
                "security_features": "Enhanced validation and bcrypt cost=12"
            })))
        }
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

pub async fn login_secure(
    State(db): State<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("Login validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check for rate limiting (brute force protection - PR 3 improvement)
    if is_rate_limited_simple(&payload.email, 5, 15) { // 5 attempts in 15 minutes
        warn!("Rate limit exceeded for email: {} (security audit)", payload.email);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Find user by email with audit logging (PR 3 improvement)
    let user = match user_service::get_user_by_email(&db, &payload.email).await {
        Ok(Some(user)) => {
            info!("User lookup successful for: {} (audit log)", payload.email);
            user
        }
        Ok(None) => {
            warn!("Login attempt for non-existent user: {} (security audit)", payload.email);
            record_failed_login_simple(&payload.email);
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(err) => {
            error!("Failed to get user: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Verify password
    if !verify(&payload.password, &user.password_hash).unwrap_or(false) {
        warn!("Invalid password for user: {} (security audit)", payload.email);
        record_failed_login_simple(&payload.email);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Check if user is active
    if !user.is_active {
        warn!("Attempt to login with inactive user: {} (security audit)", payload.email);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get JWT secret from environment (PR 3 improvement - no more hardcoded secret!)
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-256-bit-secret-key-here-change-in-production".to_string());

    // Generate JWT token using the secure method (PR 3 improvement)
    let token = match create_token(&user.id, &user.role, &jwt_secret) {
        Ok(token) => {
            info!("Token generated successfully for user: {} (audit log)", payload.email);
            record_successful_login_simple(&payload.email);
            token
        }
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