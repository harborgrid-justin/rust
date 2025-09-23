use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::{Json, Response},
};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;
use validator::Validate;

use crate::models::{CreateDocumentRequest, UpdateDocumentRequest, DocumentResponse};
use crate::services::document_service;

pub async fn list_documents(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let status = params.get("status");
    let created_by = params.get("created_by");
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(50);
    let offset = params
        .get("offset")
        .and_then(|o| o.parse::<i32>().ok())
        .unwrap_or(0);

    match document_service::list_documents(&db, status, created_by, limit, offset).await {
        Ok(documents) => {
            let responses: Vec<DocumentResponse> = documents
                .into_iter()
                .map(DocumentResponse::from)
                .collect();
            
            Ok(Json(json!({
                "documents": responses,
                "count": responses.len()
            })))
        }
        Err(err) => {
            error!("Failed to list documents: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_document(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("Document validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Extract user ID from JWT token
    let created_by = "system".to_string(); // Temporary placeholder

    match document_service::create_document(&db, payload, created_by).await {
        Ok(document) => Ok(Json(DocumentResponse::from(document))),
        Err(err) => {
            error!("Failed to create document: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_document(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    match document_service::get_document(&db, &id).await {
        Ok(Some(document)) => Ok(Json(DocumentResponse::from(document))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            error!("Failed to get document: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_document(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("Document update validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    match document_service::update_document(&db, &id, payload).await {
        Ok(Some(document)) => Ok(Json(DocumentResponse::from(document))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            error!("Failed to update document: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn upload_file(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Value>, StatusCode> {
    use std::path::Path as StdPath;
    use tokio::fs;
    use tokio::io::AsyncWriteExt;
    
    // Create upload directory if it doesn't exist
    let upload_dir = "./uploads";
    if let Err(_) = fs::create_dir_all(upload_dir).await {
        error!("Failed to create upload directory");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    // Process the uploaded file
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        if let Some(file_name) = field.file_name() {
            let file_name = file_name.to_string();
            
            // Validate file extension and type
            let allowed_extensions = ["pdf", "doc", "docx", "txt", "jpg", "jpeg", "png", "gif"];
            let extension = StdPath::new(&file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
                
            if !allowed_extensions.contains(&extension.to_lowercase().as_str()) {
                error!("Invalid file type: {}", extension);
                return Err(StatusCode::BAD_REQUEST);
            }
            
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            
            // Validate file size (5MB limit)
            const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
            if data.len() > MAX_FILE_SIZE {
                error!("File too large: {} bytes", data.len());
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
            
            // Generate unique file path
            let unique_name = format!("{}_{}", Uuid::new_v4(), file_name);
            let file_path = format!("{}/{}", upload_dir, unique_name);
            
            // Save file to disk
            match fs::File::create(&file_path).await {
                Ok(mut file) => {
                    if let Err(_) = file.write_all(&data).await {
                        error!("Failed to write file to disk");
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
                Err(_) => {
                    error!("Failed to create file");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
            
            // Update document record with file information
            let result = sqlx::query!(
                r#"
                UPDATE documents 
                SET file_path = ?1, file_name = ?2, file_size = ?3, mime_type = ?4, updated_at = ?5
                WHERE id = ?6
                "#,
                file_path,
                file_name,
                data.len() as i64,
                content_type,
                Utc::now().to_rfc3339(),
                id
            )
            .execute(&db)
            .await;
            
            match result {
                Ok(result) if result.rows_affected() > 0 => {
                    info!("File uploaded successfully: {} for document {}", file_name, id);
                    return Ok(Json(json!({
                        "message": "File uploaded successfully",
                        "document_id": id,
                        "file_name": file_name,
                        "file_size": data.len(),
                        "mime_type": content_type
                    })));
                }
                Ok(_) => {
                    // Clean up file if document not found
                    let _ = fs::remove_file(&file_path).await;
                    error!("Document not found: {}", id);
                    return Err(StatusCode::NOT_FOUND);
                }
                Err(err) => {
                    // Clean up file on database error
                    let _ = fs::remove_file(&file_path).await;
                    error!("Failed to update document: {:?}", err);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}

pub async fn download_file(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    use axum::response::Response;
    use axum::body::Body;
    use tokio::fs;
    use std::path::Path as StdPath;
    
    // Get document information from database
    let document = sqlx::query!(
        r#"
        SELECT id, title, file_path, file_name, file_size, mime_type
        FROM documents 
        WHERE id = ?1
        "#,
        id
    )
    .fetch_optional(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let document = match document {
        Some(doc) => doc,
        None => {
            error!("Document not found: {}", id);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    let file_path = match document.file_path {
        Some(path) => path,
        None => {
            error!("No file associated with document: {}", id);
            return Err(StatusCode::NOT_FOUND);
        }
    };
    
    // Check if file exists on disk
    if !StdPath::new(&file_path).exists() {
        error!("File not found on disk: {}", file_path);
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Read file from disk
    let file_content = match fs::read(&file_path).await {
        Ok(content) => content,
        Err(err) => {
            error!("Failed to read file {}: {:?}", file_path, err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Log download activity for audit
    info!("File downloaded: {} (document {})", 
          document.file_name.as_deref().unwrap_or("unknown"), 
          id);
    
    // Create response with appropriate headers
    let content_type = document.mime_type.unwrap_or("application/octet-stream".to_string());
    let file_name = document.file_name.unwrap_or("download".to_string());
    
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", file_name))
        .header("Content-Length", file_content.len().to_string())
        .header("Cache-Control", "no-cache")
        .body(Body::from(file_content))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(response)
}