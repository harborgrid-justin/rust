use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
    pub role: Option<String>, // User role for RBAC
    pub permissions: Vec<String>, // Specific permissions
}

pub fn create_token(user_id: &str, role: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        role: Some(role.to_string()),
        permissions: get_role_permissions(role),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();
    
    let token_data = decode::<Claims>(token, &key, &validation)?;
    Ok(token_data.claims)
}

pub fn extract_user_id_from_token(token: &str, secret: &str) -> Option<String> {
    match verify_token(token, secret) {
        Ok(claims) => Some(claims.sub),
        Err(_) => None,
    }
}

fn get_role_permissions(role: &str) -> Vec<String> {
    match role {
        "admin" => vec![
            "user:create".to_string(),
            "user:read".to_string(),
            "user:update".to_string(),
            "user:delete".to_string(),
            "case:create".to_string(),
            "case:read".to_string(),
            "case:update".to_string(),
            "case:delete".to_string(),
            "document:create".to_string(),
            "document:read".to_string(),
            "document:update".to_string(),
            "document:delete".to_string(),
            "system:admin".to_string(),
        ],
        "manager" => vec![
            "case:create".to_string(),
            "case:read".to_string(),
            "case:update".to_string(),
            "document:create".to_string(),
            "document:read".to_string(),
            "document:update".to_string(),
            "user:read".to_string(),
        ],
        "user" => vec![
            "case:read".to_string(),
            "case:update".to_string(),
            "document:read".to_string(),
            "document:create".to_string(),
        ],
        "readonly" => vec![
            "case:read".to_string(),
            "document:read".to_string(),
        ],
        _ => vec![],
    }
}

// Rate limiting for brute force protection
pub struct LoginAttempt {
    pub email: String,
    pub attempts: u32,
    pub last_attempt: DateTime<Utc>,
}

pub fn is_rate_limited(email: &str, max_attempts: u32, window_minutes: i64) -> bool {
    // In production, this would use Redis or a database
    // For now, this is a placeholder that would need proper implementation
    false
}

pub fn record_failed_login(email: &str) {
    // Record failed login attempt for rate limiting
    // This would be implemented with Redis or database storage
}

pub fn record_successful_login(email: &str) {
    // Clear failed login attempts on successful login
}