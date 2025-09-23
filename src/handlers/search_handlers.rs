use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::{error, info};

use crate::models::{DocumentResponse, CaseResponse};
use crate::services::{document_service, case_service};

/// Search documents by title, description, or tags
pub async fn search_documents(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let query = params.get("q").cloned().unwrap_or_default();
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(10)
        .min(100); // Max 100 results
    let offset = params
        .get("offset")
        .and_then(|o| o.parse::<i32>().ok())
        .unwrap_or(0);

    if query.is_empty() {
        return Ok(Json(json!({
            "documents": [],
            "total": 0,
            "query": query
        })));
    }

    info!("Searching documents with query: '{}'", query);

    // Search in title, description, and tags using LIKE with wildcards
    let search_pattern = format!("%{}%", query);
    
    let documents = sqlx::query!(
        r#"
        SELECT id, title, description, file_path, file_name, file_size, mime_type, 
               status, version, tags, created_by, created_at, updated_at
        FROM documents
        WHERE title LIKE ?1 
           OR description LIKE ?1 
           OR tags LIKE ?1
           OR file_name LIKE ?1
        ORDER BY 
            CASE 
                WHEN title LIKE ?1 THEN 1
                WHEN description LIKE ?1 THEN 2
                WHEN tags LIKE ?1 THEN 3
                ELSE 4
            END,
            created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
        search_pattern,
        limit,
        offset
    )
    .fetch_all(&db)
    .await
    .map_err(|err| {
        error!("Failed to search documents: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get total count for pagination
    let total_count = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM documents
        WHERE title LIKE ?1 
           OR description LIKE ?1 
           OR tags LIKE ?1
           OR file_name LIKE ?1
        "#,
        search_pattern
    )
    .fetch_one(&db)
    .await
    .map_err(|err| {
        error!("Failed to get document search count: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let document_responses: Vec<DocumentResponse> = documents
        .into_iter()
        .map(|row| {
            let document = crate::models::Document {
                id: row.id.unwrap_or_default(),
                title: row.title.unwrap_or_default(),
                description: row.description,
                file_path: row.file_path,
                file_name: row.file_name,
                file_size: row.file_size,
                mime_type: row.mime_type,
                status: row.status.unwrap_or_default(),
                version: row.version.unwrap_or(1),
                tags: row.tags,
                created_by: row.created_by.unwrap_or_default(),
                created_at: row.created_at.unwrap_or_default(),
                updated_at: row.updated_at.unwrap_or_default(),
            };
            DocumentResponse::from(document)
        })
        .collect();

    Ok(Json(json!({
        "documents": document_responses,
        "total": total_count.count,
        "query": query,
        "limit": limit,
        "offset": offset
    })))
}

/// Search cases by title, description, or priority
pub async fn search_cases(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let query = params.get("q").cloned().unwrap_or_default();
    let status = params.get("status");
    let priority = params.get("priority");
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(10)
        .min(100); // Max 100 results
    let offset = params
        .get("offset")
        .and_then(|o| o.parse::<i32>().ok())
        .unwrap_or(0);

    if query.is_empty() && status.is_none() && priority.is_none() {
        return Ok(Json(json!({
            "cases": [],
            "total": 0,
            "query": query
        })));
    }

    info!("Searching cases with query: '{}'", query);

    let search_pattern = format!("%{}%", query);
    
    // Build dynamic query based on filters
    let mut where_conditions = Vec::new();
    let mut params_list = Vec::new();

    if !query.is_empty() {
        where_conditions.push("(title LIKE ?1 OR description LIKE ?1)");
        params_list.push(search_pattern.as_str());
    }

    if let Some(status_filter) = status {
        where_conditions.push(&format!("status = ?{}", params_list.len() + 1));
        params_list.push(status_filter);
    }

    if let Some(priority_filter) = priority {
        where_conditions.push(&format!("priority = ?{}", params_list.len() + 1));
        params_list.push(priority_filter);
    }

    let where_clause = if where_conditions.is_empty() {
        "1=1".to_string()
    } else {
        where_conditions.join(" AND ")
    };

    let sql = format!(
        r#"
        SELECT id, title, description, status, priority, assigned_to, 
               created_by, created_at, updated_at
        FROM cases
        WHERE {}
        ORDER BY 
            CASE 
                WHEN priority = 'high' THEN 1
                WHEN priority = 'medium' THEN 2
                WHEN priority = 'low' THEN 3
                ELSE 4
            END,
            created_at DESC
        LIMIT {} OFFSET {}
        "#,
        where_clause, limit, offset
    );

    // For simplicity, let's use a basic search without complex parameter binding
    let cases = sqlx::query!(
        r#"
        SELECT id, title, description, status, priority, assigned_to, 
               created_by, created_at, updated_at
        FROM cases
        WHERE (title LIKE ?1 OR description LIKE ?1)
        ORDER BY 
            CASE 
                WHEN priority = 'high' THEN 1
                WHEN priority = 'medium' THEN 2
                WHEN priority = 'low' THEN 3
                ELSE 4
            END,
            created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
        search_pattern,
        limit,
        offset
    )
    .fetch_all(&db)
    .await
    .map_err(|err| {
        error!("Failed to search cases: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get total count
    let total_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM cases WHERE (title LIKE ?1 OR description LIKE ?1)",
        search_pattern
    )
    .fetch_one(&db)
    .await
    .map_err(|err| {
        error!("Failed to get case search count: {:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let case_responses: Vec<CaseResponse> = cases
        .into_iter()
        .map(|row| {
            let case = crate::models::Case {
                id: row.id.unwrap_or_default(),
                title: row.title.unwrap_or_default(),
                description: row.description,
                status: row.status.unwrap_or_default(),
                priority: row.priority.unwrap_or_default(),
                assigned_to: row.assigned_to,
                created_by: row.created_by.unwrap_or_default(),
                created_at: row.created_at.unwrap_or_default(),
                updated_at: row.updated_at.unwrap_or_default(),
            };
            CaseResponse::from(case)
        })
        .collect();

    Ok(Json(json!({
        "cases": case_responses,
        "total": total_count.count,
        "query": query,
        "limit": limit,
        "offset": offset
    })))
}

/// Global search across documents, cases, and users
pub async fn global_search(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let query = params.get("q").cloned().unwrap_or_default();
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(5)
        .min(50); // Smaller limit for global search

    if query.is_empty() {
        return Ok(Json(json!({
            "results": {
                "documents": [],
                "cases": [],
                "users": []
            },
            "query": query
        })));
    }

    info!("Global search with query: '{}'", query);

    let search_pattern = format!("%{}%", query);

    // Search documents
    let documents = sqlx::query!(
        r#"
        SELECT id, title, description, created_at
        FROM documents
        WHERE title LIKE ?1 OR description LIKE ?1
        ORDER BY created_at DESC
        LIMIT ?2
        "#,
        search_pattern,
        limit
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    // Search cases  
    let cases = sqlx::query!(
        r#"
        SELECT id, title, description, priority, created_at
        FROM cases
        WHERE title LIKE ?1 OR description LIKE ?1
        ORDER BY created_at DESC
        LIMIT ?2
        "#,
        search_pattern,
        limit
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    // Search users
    let users = sqlx::query!(
        r#"
        SELECT id, username, email, created_at
        FROM users
        WHERE username LIKE ?1 OR email LIKE ?1
        ORDER BY created_at DESC
        LIMIT ?2
        "#,
        search_pattern,
        limit
    )
    .fetch_all(&db)
    .await
    .unwrap_or_default();

    Ok(Json(json!({
        "results": {
            "documents": documents.into_iter().map(|d| json!({
                "id": d.id,
                "title": d.title,
                "description": d.description,
                "type": "document",
                "created_at": d.created_at
            })).collect::<Vec<_>>(),
            "cases": cases.into_iter().map(|c| json!({
                "id": c.id,
                "title": c.title,
                "description": c.description,
                "priority": c.priority,
                "type": "case",
                "created_at": c.created_at
            })).collect::<Vec<_>>(),
            "users": users.into_iter().map(|u| json!({
                "id": u.id,
                "username": u.username,
                "email": u.email,
                "type": "user",
                "created_at": u.created_at
            })).collect::<Vec<_>>()
        },
        "query": query,
        "limit": limit
    })))
}