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
    let row = sqlx::query!(
        "SELECT id, email, username, password_hash, full_name, role, is_active, created_at, updated_at FROM users WHERE email = ?1 LIMIT 1",
        email
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(User {
            id: row.id,
            email: row.email,
            username: row.username,
            password_hash: row.password_hash,
            full_name: row.full_name,
            role: row.role,
            is_active: row.is_active != 0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })),
        None => Ok(None),
    }
}

pub async fn get_user_by_id(db: &SqlitePool, id: &str) -> Result<Option<User>> {
    let row = sqlx::query!(
        "SELECT id, email, username, password_hash, full_name, role, is_active, created_at, updated_at FROM users WHERE id = ?1 LIMIT 1",
        id
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(User {
            id: row.id,
            email: row.email,
            username: row.username,
            password_hash: row.password_hash,
            full_name: row.full_name,
            role: row.role,
            is_active: row.is_active != 0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })),
        None => Ok(None),
    }
}