use anyhow::Result;
use sqlx::SqlitePool;
use tracing::info;

use crate::models::{User, UserRole};

pub async fn create_user(
    db: &SqlitePool,
    email: String,
    username: String,
    password_hash: String,
    full_name: Option<String>,
    role: UserRole,
) -> Result<User> {
    let user = User::new(email, username, password_hash, full_name, role);

        sqlx::query!(
        r#"
        INSERT INTO users (id, email, username, password_hash, full_name, role, is_active, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        user.id,
        user.email,
        user.username,
        user.password_hash,
        user.full_name,
        user.role,
        user.is_active,
        user.created_at,
        user.updated_at
    )
    .execute(db)
    .await?;

    info!("Created user: {}", user.email);
    Ok(user)
}

pub async fn get_user_by_email(db: &SqlitePool, email: &str) -> Result<Option<User>> {
    let row = sqlx::query_as!(
        User,
        "SELECT id, email, username, password_hash, full_name, role, is_active, created_at, updated_at FROM users WHERE email = ?1 LIMIT 1",
        email
    )
    .fetch_optional(db)
    .await?;

    Ok(row)
}

pub async fn get_user_by_id(db: &SqlitePool, id: &str) -> Result<Option<User>> {
    let row = sqlx::query_as!(
        User,
        "SELECT id, email, username, password_hash, full_name, role, is_active, created_at, updated_at FROM users WHERE id = ?1 LIMIT 1",
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(row)
}