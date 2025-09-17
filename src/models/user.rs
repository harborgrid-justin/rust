use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub role: String, // Changed to String for SQLite compatibility
    pub is_active: bool,
    pub created_at: String, // Changed to String for SQLite compatibility
    pub updated_at: String, // Changed to String for SQLite compatibility
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Manager,
    User,
    ReadOnly,
}

impl From<String> for UserRole {
    fn from(s: String) -> Self {
        match s.as_str() {
            "admin" => UserRole::Admin,
            "manager" => UserRole::Manager,
            "user" => UserRole::User,
            "readonly" => UserRole::ReadOnly,
            _ => UserRole::User,
        }
    }
}

impl From<UserRole> for String {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::Admin => "admin".to_string(),
            UserRole::Manager => "manager".to_string(),
            UserRole::User => "user".to_string(),
            UserRole::ReadOnly => "readonly".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(length(min = 8))]
    pub password: String,
    
    pub full_name: Option<String>,
    pub role: Option<UserRole>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub username: String,
    pub full_name: Option<String>,
    pub role: UserRole,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            full_name: user.full_name,
            role: UserRole::from(user.role),
        }
    }
}

impl User {
    pub fn new(email: String, username: String, password_hash: String, full_name: Option<String>, role: UserRole) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            username,
            password_hash,
            full_name,
            role: String::from(role),
            is_active: true,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}