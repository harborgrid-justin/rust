use anyhow::Result;
use sqlx::SqlitePool;
use tracing::info;
use chrono::Utc;

use crate::models::{
    CaseTemplate, CaseWorkflow, CaseCustomField, CreateCaseTemplateRequest,
    UpdateWorkflowStepRequest, SetCustomFieldRequest, CaseTemplateResponse,
    CaseWorkflowResponse, WorkflowStatus, CustomFieldType, WorkflowStep
};

pub async fn create_case_template(
    db: &SqlitePool,
    request: CreateCaseTemplateRequest,
    created_by: String,
) -> Result<CaseTemplate> {
    let template = CaseTemplate::new(
        request.name,
        request.description,
        request.default_priority.unwrap_or("medium".to_string()),
        request.default_assignee,
        request.workflow_steps,
        request.custom_fields,
        created_by,
    );

    sqlx::query!(
        r#"
        INSERT INTO case_templates (id, name, description, default_priority, default_assignee, 
                                   workflow_steps, custom_fields, created_by, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
        template.id,
        template.name,
        template.description,
        template.default_priority,
        template.default_assignee,
        template.workflow_steps,
        template.custom_fields,
        template.created_by,
        template.created_at,
        template.updated_at
    )
    .execute(db)
    .await?;

    info!("Created case template: {} ({})", template.name, template.id);
    Ok(template)
}

pub async fn get_case_template(db: &SqlitePool, id: &str) -> Result<Option<CaseTemplateResponse>> {
    let row = sqlx::query!(
        r#"
        SELECT id, name, description, default_priority, default_assignee, 
               workflow_steps, custom_fields, created_by, created_at, updated_at
        FROM case_templates
        WHERE id = ?1
        "#,
        id
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => {
            let template = CaseTemplate {
                id: row.id.unwrap_or_default(),
                name: row.name.unwrap_or_default(),
                description: row.description,
                default_priority: row.default_priority.unwrap_or_default(),
                default_assignee: row.default_assignee,
                workflow_steps: row.workflow_steps,
                custom_fields: row.custom_fields,
                created_by: row.created_by.unwrap_or_default(),
                created_at: row.created_at.unwrap_or_default(),
                updated_at: row.updated_at.unwrap_or_default(),
            };
            Ok(Some(CaseTemplateResponse::from(template)))
        }
        None => Ok(None),
    }
}

pub async fn list_case_templates(
    db: &SqlitePool,
    limit: i32,
    offset: i32,
) -> Result<Vec<CaseTemplateResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, name, description, default_priority, default_assignee, 
               workflow_steps, custom_fields, created_by, created_at, updated_at
        FROM case_templates
        ORDER BY created_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
        limit,
        offset
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let template = CaseTemplate {
                id: row.id.unwrap_or_default(),
                name: row.name.unwrap_or_default(),
                description: row.description,
                default_priority: row.default_priority.unwrap_or_default(),
                default_assignee: row.default_assignee,
                workflow_steps: row.workflow_steps,
                custom_fields: row.custom_fields,
                created_by: row.created_by.unwrap_or_default(),
                created_at: row.created_at.unwrap_or_default(),
                updated_at: row.updated_at.unwrap_or_default(),
            };
            CaseTemplateResponse::from(template)
        })
        .collect())
}

pub async fn delete_case_template(db: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query!("DELETE FROM case_templates WHERE id = ?1", id)
        .execute(db)
        .await?;

    info!("Deleted case template {}", id);
    Ok(())
}

pub async fn create_case_workflow_from_template(
    db: &SqlitePool,
    case_id: &str,
    template_id: &str,
) -> Result<Vec<CaseWorkflow>> {
    // Get template workflow steps
    let template = sqlx::query!(
        "SELECT workflow_steps FROM case_templates WHERE id = ?1",
        template_id
    )
    .fetch_optional(db)
    .await?;

    if let Some(template) = template {
        if let Some(workflow_json) = template.workflow_steps {
            let workflow_steps: Vec<WorkflowStep> = serde_json::from_str(&workflow_json)?;
            let mut created_workflows = Vec::new();

            for step in workflow_steps {
                let workflow = CaseWorkflow::new(
                    case_id.to_string(),
                    step.name,
                    step.order,
                    None, // No default assignment
                );

                sqlx::query!(
                    r#"
                    INSERT INTO case_workflows (id, case_id, step_name, step_order, status, 
                                               assigned_to, completed_by, completed_at, notes, 
                                               created_at, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                    "#,
                    workflow.id,
                    workflow.case_id,
                    workflow.step_name,
                    workflow.step_order,
                    workflow.status,
                    workflow.assigned_to,
                    workflow.completed_by,
                    workflow.completed_at,
                    workflow.notes,
                    workflow.created_at,
                    workflow.updated_at
                )
                .execute(db)
                .await?;

                created_workflows.push(workflow);
            }

            info!("Created {} workflow steps for case {}", created_workflows.len(), case_id);
            return Ok(created_workflows);
        }
    }

    Ok(vec![])
}

pub async fn get_case_workflows(db: &SqlitePool, case_id: &str) -> Result<Vec<CaseWorkflowResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, case_id, step_name, step_order, status, assigned_to, 
               completed_by, completed_at, notes, created_at, updated_at
        FROM case_workflows
        WHERE case_id = ?1
        ORDER BY step_order ASC
        "#,
        case_id
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let workflow = CaseWorkflow {
                id: row.id.unwrap_or_default(),
                case_id: row.case_id.unwrap_or_default(),
                step_name: row.step_name.unwrap_or_default(),
                step_order: row.step_order.unwrap_or(0),
                status: row.status.unwrap_or_default(),
                assigned_to: row.assigned_to,
                completed_by: row.completed_by,
                completed_at: row.completed_at,
                notes: row.notes,
                created_at: row.created_at.unwrap_or_default(),
                updated_at: row.updated_at.unwrap_or_default(),
            };
            CaseWorkflowResponse::from(workflow)
        })
        .collect())
}

pub async fn update_workflow_step(
    db: &SqlitePool,
    workflow_id: &str,
    request: UpdateWorkflowStepRequest,
    updated_by: String,
) -> Result<Option<CaseWorkflowResponse>> {
    let now = Utc::now().to_rfc3339();
    
    let mut completed_by = None;
    let mut completed_at = None;
    
    if let Some(WorkflowStatus::Completed) = request.status {
        completed_by = Some(updated_by);
        completed_at = Some(now.clone());
    }

    sqlx::query!(
        r#"
        UPDATE case_workflows 
        SET status = COALESCE(?1, status),
            assigned_to = COALESCE(?2, assigned_to),
            notes = COALESCE(?3, notes),
            completed_by = COALESCE(?4, completed_by),
            completed_at = COALESCE(?5, completed_at),
            updated_at = ?6
        WHERE id = ?7
        "#,
        request.status.map(String::from),
        request.assigned_to,
        request.notes,
        completed_by,
        completed_at,
        now,
        workflow_id
    )
    .execute(db)
    .await?;

    // Get the updated workflow
    let row = sqlx::query!(
        r#"
        SELECT id, case_id, step_name, step_order, status, assigned_to, 
               completed_by, completed_at, notes, created_at, updated_at
        FROM case_workflows
        WHERE id = ?1
        "#,
        workflow_id
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => {
            let workflow = CaseWorkflow {
                id: row.id.unwrap_or_default(),
                case_id: row.case_id.unwrap_or_default(),
                step_name: row.step_name.unwrap_or_default(),
                step_order: row.step_order.unwrap_or(0),
                status: row.status.unwrap_or_default(),
                assigned_to: row.assigned_to,
                completed_by: row.completed_by,
                completed_at: row.completed_at,
                notes: row.notes,
                created_at: row.created_at.unwrap_or_default(),
                updated_at: row.updated_at.unwrap_or_default(),
            };
            Ok(Some(CaseWorkflowResponse::from(workflow)))
        }
        None => Ok(None),
    }
}

pub async fn set_case_custom_field(
    db: &SqlitePool,
    case_id: &str,
    field_name: &str,
    field_type: CustomFieldType,
    request: SetCustomFieldRequest,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        INSERT INTO case_custom_fields (id, case_id, field_name, field_type, field_value, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT (case_id, field_name) 
        DO UPDATE SET field_value = ?5, updated_at = ?7
        "#,
        uuid::Uuid::new_v4().to_string(),
        case_id,
        field_name,
        String::from(field_type),
        request.field_value,
        now.clone(),
        now
    )
    .execute(db)
    .await?;

    info!("Set custom field {} for case {}", field_name, case_id);
    Ok(())
}

pub async fn get_case_custom_fields(
    db: &SqlitePool,
    case_id: &str,
) -> Result<Vec<CaseCustomField>> {
    let rows = sqlx::query!(
        r#"
        SELECT id, case_id, field_name, field_type, field_value, created_at, updated_at
        FROM case_custom_fields
        WHERE case_id = ?1
        ORDER BY field_name ASC
        "#,
        case_id
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| CaseCustomField {
            id: row.id.unwrap_or_default(),
            case_id: row.case_id.unwrap_or_default(),
            field_name: row.field_name.unwrap_or_default(),
            field_type: row.field_type.unwrap_or_default(),
            field_value: row.field_value,
            created_at: row.created_at.unwrap_or_default(),
            updated_at: row.updated_at.unwrap_or_default(),
        })
        .collect())
}