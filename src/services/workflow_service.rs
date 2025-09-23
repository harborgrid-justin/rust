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
        if let Some(workflow_json) = &template.workflow_steps {
            if let Ok(workflow_steps) = serde_json::from_str::<Vec<WorkflowStep>>(workflow_json) {
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
    
    // Get current workflow state for validation
    let current_workflow = sqlx::query!(
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
    
    let current_workflow = match current_workflow {
        Some(w) => w,
        None => return Ok(None),
    };
    
    let current_status = WorkflowStatus::from(current_workflow.status.unwrap_or_default());
    
    // Validate workflow state transitions
    if let Some(new_status) = &request.status {
        if !is_valid_workflow_transition(&current_status, new_status) {
            info!("Invalid workflow transition from {:?} to {:?} for workflow {}", 
                  current_status, new_status, workflow_id);
            return Err(anyhow::anyhow!("Invalid workflow state transition"));
        }
        
        // Business rule: Can't complete a step without being assigned
        if *new_status == WorkflowStatus::Completed {
            let assigned_to = request.assigned_to.as_ref()
                .or(current_workflow.assigned_to.as_ref());
                
            if assigned_to.is_none() {
                return Err(anyhow::anyhow!("Cannot complete workflow step without assignment"));
            }
        }
        
        // Business rule: Check prerequisites for this workflow step
        if *new_status == WorkflowStatus::InProgress || *new_status == WorkflowStatus::Completed {
            let prerequisite_check = check_workflow_prerequisites(
                db, 
                &current_workflow.case_id.unwrap_or_default(), 
                current_workflow.step_order.unwrap_or(0)
            ).await?;
            
            if !prerequisite_check {
                return Err(anyhow::anyhow!("Prerequisite workflow steps not completed"));
            }
        }
    }
    
    let mut completed_by = None;
    let mut completed_at = None;
    
    // Set completion fields if status is completed
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
                step_order: row.step_order.unwrap_or(0) as i32,
                status: row.status.unwrap_or_default(),
                assigned_to: row.assigned_to,
                completed_by: row.completed_by,
                completed_at: row.completed_at,
                notes: row.notes,
                created_at: row.created_at.unwrap_or_default(),
                updated_at: row.updated_at.unwrap_or_default(),
            };
            
            // Log workflow step update for audit
            info!("Workflow step updated: {} ({})", workflow.step_name, workflow.id);
            
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
    let id = uuid::Uuid::new_v4().to_string();
    let field_type_string = String::from(field_type);

    sqlx::query!(
        r#"
        INSERT INTO case_custom_fields (id, case_id, field_name, field_type, field_value, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT (case_id, field_name) 
        DO UPDATE SET field_value = ?5, updated_at = ?7
        "#,
        id,
        case_id,
        field_name,
        field_type_string,
        request.field_value,
        now,
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

// Helper functions for workflow validation

/// Validates if a workflow status transition is allowed
fn is_valid_workflow_transition(from: &WorkflowStatus, to: &WorkflowStatus) -> bool {
    use WorkflowStatus::*;
    
    match (from, to) {
        // From Pending
        (Pending, InProgress) => true,
        (Pending, Skipped) => true,
        // From InProgress  
        (InProgress, Completed) => true,
        (InProgress, Skipped) => true,
        (InProgress, Pending) => true, // Can go back if needed
        // From Completed
        (Completed, InProgress) => true, // Can reopen if needed
        // From Skipped
        (Skipped, Pending) => true,
        (Skipped, InProgress) => true,
        // Same status is always valid
        (from, to) if from == to => true,
        // All other transitions are invalid
        _ => false,
    }
}

/// Checks if prerequisite workflow steps are completed
async fn check_workflow_prerequisites(
    db: &SqlitePool, 
    case_id: &str, 
    current_step_order: i32
) -> Result<bool> {
    // Check if all previous steps (lower step_order) are completed or skipped
    let incomplete_prerequisites = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM case_workflows
        WHERE case_id = ?1 
          AND step_order < ?2 
          AND status NOT IN ('completed', 'skipped')
        "#,
        case_id,
        current_step_order
    )
    .fetch_one(db)
    .await?;
    
    Ok(incomplete_prerequisites.count == 0)
}