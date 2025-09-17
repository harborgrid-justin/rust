use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use tracing::info;

use crate::models::{Document, CreateDocumentRequest, UpdateDocumentRequest, DocumentStatus};

pub async fn create_document(
    db: &SqlitePool,
    request: CreateDocumentRequest,
    created_by: String,
) -> Result<Document> {
    let document = Document::new(
        request.title,
        request.description,
        created_by,
        request.tags,
    );

    let tags_json = document.tags.as_deref();

    sqlx::query!(
        r#"
        INSERT INTO documents (id, title, description, file_path, file_name, file_size, 
                             mime_type, status, version, tags, created_by, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
        document.id,
        document.title,
        document.description,
        document.file_path,
        document.file_name,
        document.file_size,
        document.mime_type,
        document.status,
        document.version,
        tags_json,
        document.created_by,
        document.created_at,
        document.updated_at
    )
    .execute(db)
    .await?;

    info!("Created document: {}", document.id);
    Ok(document)
}

pub async fn get_document(db: &SqlitePool, id: &str) -> Result<Option<Document>> {
    let row = sqlx::query!(
        "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents WHERE id = ?1 LIMIT 1",
        id
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(Document {
            id: row.id,
            title: row.title,
            description: row.description,
            file_path: row.file_path,
            file_name: row.file_name,
            file_size: row.file_size,
            mime_type: row.mime_type,
            status: row.status,
            version: row.version as i32,
            tags: row.tags,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })),
        None => Ok(None),
    }
}

pub async fn list_documents(
    db: &SqlitePool,
    status: Option<&String>,
    created_by: Option<&String>,
    limit: i32,
    offset: i32,
) -> Result<Vec<Document>> {
    let rows = match (status, created_by) {
        (Some(status), Some(created_by)) => {
            sqlx::query!(
                "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents WHERE status = ?1 AND created_by = ?2 ORDER BY created_at DESC LIMIT ?3 OFFSET ?4",
                status, created_by, limit, offset
            )
            .fetch_all(db)
            .await?
        }
        (Some(status), None) => {
            sqlx::query!(
                "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
                status, limit, offset
            )
            .fetch_all(db)
            .await?
        }
        (None, Some(created_by)) => {
            sqlx::query!(
                "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents WHERE created_by = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
                created_by, limit, offset
            )
            .fetch_all(db)
            .await?
        }
        (None, None) => {
            sqlx::query!(
                "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
                limit, offset
            )
            .fetch_all(db)
            .await?
        }
    };

    let documents = rows
        .into_iter()
        .map(|row| Document {
            id: row.id,
            title: row.title,
            description: row.description,
            file_path: row.file_path,
            file_name: row.file_name,
            file_size: row.file_size,
            mime_type: row.mime_type,
            status: row.status,
            version: row.version as i32,
            tags: row.tags,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
        .collect();

    Ok(documents)
}

pub async fn update_document(
    db: &SqlitePool,
    id: &str,
    request: UpdateDocumentRequest,
) -> Result<Option<Document>> {
    let updated_at = Utc::now().to_rfc3339();

    // For simplicity, let's use a more straightforward approach
    let tags_json = request.tags.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default());
    let status_str = request.status.map(String::from);
    
    sqlx::query!(
        r#"
        UPDATE documents 
        SET title = COALESCE(?1, title),
            description = COALESCE(?2, description),
            status = COALESCE(?3, status),
            tags = COALESCE(?4, tags),
            updated_at = ?5
        WHERE id = ?6
        "#,
        request.title,
        request.description,
        status_str,
        tags_json,
        updated_at,
        id
    )
    .execute(db)
    .await?;

    info!("Updated document: {}", id);
    get_document(db, id).await
}