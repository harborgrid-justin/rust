use axum::{extract::State, http::StatusCode, response::Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde_json::{json, Value};
use tracing::{error, info, warn};
use validator::Validate;

use crate::app_state::AppState;
use crate::models::{CreateUserRequest, LoginRequest, LoginResponse, UserRole};
use crate::services::user_service;
use crate::utils::auth::{create_token, is_rate_limited, record_failed_login, record_successful_login};

fn has_password_complexity(password: &str) -> bool {
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    
    has_upper && has_lower && has_digit && has_special
}

pub async fn register(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("User registration validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Enhanced validation for business logic
    if payload.password.len() < 8 {
        error!("Password too short: minimum 8 characters required");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check password complexity
    if !has_password_complexity(&payload.password) {
        error!("Password does not meet complexity requirements");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Hash password with higher cost for better security
    let password_hash = match hash(&payload.password, 12) { // Increased from DEFAULT_COST
        Ok(hash) => hash,
        Err(err) => {
            error!("Failed to hash password: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let role = payload.role.unwrap_or(UserRole::User);

    match user_service::create_user(
        &app_state.db,
        payload.email,
        payload.username,
        password_hash,
        payload.full_name,
        role,
    )
    .await
    {
        Ok(user) => {
            info!("User registered successfully: {}", user.email);
            Ok(Json(json!({
                "message": "User registered successfully",
                "user": {
                    "id": user.id,
                    "email": user.email,
                    "username": user.username,
                    "full_name": user.full_name,
                    "role": user.role
                }
            })))
        },
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
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("Login validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check for rate limiting (brute force protection)
    if is_rate_limited(&payload.email, 5, 15) { // 5 attempts in 15 minutes
        warn!("Rate limit exceeded for email: {}", payload.email);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Find user by email with audit logging
    let user = match user_service::get_user_by_email(&app_state.db, &payload.email).await {
        Ok(Some(user)) => {
            info!("User lookup successful for: {}", payload.email);
            user
        }
        Ok(None) => {
            warn!("Login attempt for non-existent user: {}", payload.email);
            record_failed_login(&payload.email);
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(err) => {
            error!("Failed to get user: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Verify password
    if !verify(&payload.password, &user.password_hash).unwrap_or(false) {
        warn!("Invalid password for user: {}", payload.email);
        record_failed_login(&payload.email);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Check if user is active
    if !user.is_active {
        warn!("Attempt to login with inactive user: {}", payload.email);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate JWT token using the secure method
    let token = match create_token(&user.id, &user.role, &app_state.config.jwt_secret) {
        Ok(token) => {
            info!("Token generated successfully for user: {}", payload.email);
            record_successful_login(&payload.email);
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