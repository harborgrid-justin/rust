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

use crate::models::{CreateCaseTemplateRequest, UpdateWorkflowStepRequest, SetCustomFieldRequest, CustomFieldType};
use crate::services::workflow_service;

pub async fn create_case_template(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateCaseTemplateRequest>,
) -> Result<Json<Value>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("Case template validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Extract user ID from JWT token
    let created_by = "system".to_string(); // Temporary placeholder

    match workflow_service::create_case_template(&db, payload, created_by).await {
        Ok(template) => Ok(Json(json!({
            "message": "Case template created successfully",
            "template_id": template.id,
            "name": template.name
        }))),
        Err(err) => {
            error!("Failed to create case template: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_case_template(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match workflow_service::get_case_template(&db, &id).await {
        Ok(Some(template)) => Ok(Json(serde_json::to_value(template).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            error!("Failed to get case template: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_case_templates(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20);
    let offset = params
        .get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    match workflow_service::list_case_templates(&db, limit, offset).await {
        Ok(templates) => Ok(Json(json!({
            "templates": templates,
            "count": templates.len()
        }))),
        Err(err) => {
            error!("Failed to list case templates: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_case_template(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match workflow_service::delete_case_template(&db, &id).await {
        Ok(_) => Ok(Json(json!({
            "message": "Case template deleted successfully",
            "template_id": id
        }))),
        Err(err) => {
            error!("Failed to delete case template: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_case_workflows(
    State(db): State<SqlitePool>,
    Path(case_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match workflow_service::get_case_workflows(&db, &case_id).await {
        Ok(workflows) => Ok(Json(json!({
            "workflows": workflows,
            "count": workflows.len()
        }))),
        Err(err) => {
            error!("Failed to get case workflows: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_workflow_step(
    State(db): State<SqlitePool>,
    Path(workflow_id): Path<String>,
    Json(payload): Json<UpdateWorkflowStepRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token
    let updated_by = "system".to_string(); // Temporary placeholder

    match workflow_service::update_workflow_step(&db, &workflow_id, payload, updated_by).await {
        Ok(Some(workflow)) => Ok(Json(serde_json::to_value(workflow).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            error!("Failed to update workflow step: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_case_from_template(
    State(db): State<SqlitePool>,
    Path((case_id, template_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    match workflow_service::create_case_workflow_from_template(&db, &case_id, &template_id).await {
        Ok(workflows) => Ok(Json(json!({
            "message": "Case workflows created from template",
            "case_id": case_id,
            "template_id": template_id,
            "workflows_created": workflows.len()
        }))),
        Err(err) => {
            error!("Failed to create case from template: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_case_custom_fields(
    State(db): State<SqlitePool>,
    Path(case_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match workflow_service::get_case_custom_fields(&db, &case_id).await {
        Ok(fields) => Ok(Json(json!({
            "custom_fields": fields,
            "count": fields.len()
        }))),
        Err(err) => {
            error!("Failed to get case custom fields: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn set_case_custom_field(
    State(db): State<SqlitePool>,
    Path((case_id, field_name)): Path<(String, String)>,
    Json(payload): Json<SetCustomFieldRequest>,
) -> Result<Json<Value>, StatusCode> {
    // For simplicity, default to text field type
    let field_type = CustomFieldType::Text;

    match workflow_service::set_case_custom_field(&db, &case_id, &field_name, field_type, payload).await {
        Ok(_) => Ok(Json(json!({
            "message": "Custom field set successfully",
            "case_id": case_id,
            "field_name": field_name
        }))),
        Err(err) => {
            error!("Failed to set case custom field: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}