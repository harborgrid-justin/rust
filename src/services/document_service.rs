use anyhow::Result;
use sqlx::SqlitePool;
use tracing::info;

use crate::models::{Document, DocumentStatus, CreateDocumentRequest, DocumentResponse, UpdateDocumentRequest};

pub async fn create_document(
    db: &SqlitePool,
    request: CreateDocumentRequest,
    created_by: String,
) -> Result<Document> {
    let document = Document::new(request.title, request.description, created_by, request.tags);

    sqlx::query!(
        r#"
        INSERT INTO documents (id, title, description, file_path, file_name, file_size, mime_type, 
                             status, version, tags, created_by, created_at, updated_at)
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
        document.tags,
        document.created_by,
        document.created_at,
        document.updated_at
    )
    .execute(db)
    .await?;

    info!("Created document: {} ({})", document.title, document.id);
    Ok(document)
}

pub async fn get_document(db: &SqlitePool, id: &str) -> Result<Option<Document>> {
    let document = sqlx::query_as!(
        Document,
        "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents WHERE id = ?1",
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(document)
}

pub async fn list_documents(
    db: &SqlitePool,
    status: Option<&String>,
    created_by: Option<&String>,
    limit: i32,
    offset: i32,
) -> Result<Vec<Document>> {
    let documents = match (status, created_by) {
        (Some(status), Some(created_by)) => {
            sqlx::query_as!(
                Document,
                "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents WHERE status = ?1 AND created_by = ?2 ORDER BY created_at DESC LIMIT ?3 OFFSET ?4",
                status, created_by, limit, offset
            )
            .fetch_all(db)
            .await?
        }
        (Some(status), None) => {
            sqlx::query_as!(
                Document,
                "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
                status, limit, offset
            )
            .fetch_all(db)
            .await?
        }
        (None, Some(created_by)) => {
            sqlx::query_as!(
                Document,
                "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents WHERE created_by = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
                created_by, limit, offset
            )
            .fetch_all(db)
            .await?
        }
        (None, None) => {
            sqlx::query_as!(
                Document,
                "SELECT id, title, description, file_path, file_name, file_size, mime_type, status, version, tags, created_by, created_at, updated_at FROM documents ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
                limit, offset
            )
            .fetch_all(db)
            .await?
        }
    };

    Ok(documents)
}

pub async fn update_document(
    db: &SqlitePool,
    id: &str,
    request: UpdateDocumentRequest,
    updated_by: String,
) -> Result<Option<Document>> {
    let now = chrono::Utc::now().to_rfc3339();
    let status_str = request.status.map(String::from);
    let tags_json = request.tags.map(|tags| serde_json::to_string(&tags).unwrap_or_default());

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
        now,
        id
    )
    .execute(db)
    .await?;

    // Return the updated document
    get_document(db, id).await
}

pub async fn delete_document(db: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query!("DELETE FROM documents WHERE id = ?1", id)
        .execute(db)
        .await?;

    info!("Deleted document {}", id);
    Ok(())
}