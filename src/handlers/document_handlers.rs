use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::error;
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
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement file upload functionality
    Ok(Json(json!({
        "message": "File upload functionality not yet implemented",
        "document_id": id
    })))
}

pub async fn download_file(
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement file download functionality
    Ok(Json(json!({
        "message": "File download functionality not yet implemented",
        "document_id": id
    })))
}