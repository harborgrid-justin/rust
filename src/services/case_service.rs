use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use tracing::info;

use crate::models::{
    Case, CaseDocument, CaseHistory, CasePriority, CaseStatus, CreateCaseRequest, Document,
    UpdateCaseRequest,
};

pub async fn create_case(
    db: &SqlitePool,
    request: CreateCaseRequest,
    created_by: String,
) -> Result<Case> {
    let priority = request.priority.unwrap_or(CasePriority::Medium);
    
    let case = Case::new(
        request.title,
        request.description,
        priority,
        request.assigned_to.clone(),
        created_by.clone(),
        request.due_date,
    );

    sqlx::query!(
        r#"
        INSERT INTO cases (id, title, description, status, priority, assigned_to, 
                          created_by, due_date, closed_at, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        case.id,
        case.title,
        case.description,
        case.status,
        case.priority,
        case.assigned_to,
        case.created_by,
        case.due_date,
        case.closed_at,
        case.created_at,
        case.updated_at
    )
    .execute(db)
    .await?;

    // Create case history entry
    let history = CaseHistory::new(
        case.id.clone(),
        "Case created".to_string(),
        Some(format!("Case created by {}", created_by)),
        created_by,
    );

    sqlx::query!(
        "INSERT INTO case_history (id, case_id, action, details, performed_by, performed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        history.id,
        history.case_id,
        history.action,
        history.details,
        history.performed_by,
        history.performed_at
    )
    .execute(db)
    .await?;

    info!("Created case: {}", case.id);
    Ok(case)
}

pub async fn get_case(db: &SqlitePool, id: &str) -> Result<Option<Case>> {
    let case = sqlx::query_as!(
        Case,
        "SELECT * FROM cases WHERE id = ?1 LIMIT 1",
        id
    )
    .fetch_optional(db)
    .await?;

    Ok(case)
}

pub async fn list_cases(
    db: &SqlitePool,
    status: Option<&String>,
    assigned_to: Option<&String>,
    limit: i32,
    offset: i32,
) -> Result<Vec<Case>> {
    let cases = match (status, assigned_to) {
        (Some(status), Some(assigned_to)) => {
            sqlx::query_as!(
                Case,
                "SELECT * FROM cases WHERE status = ?1 AND assigned_to = ?2 
                 ORDER BY created_at DESC LIMIT ?3 OFFSET ?4",
                status, assigned_to, limit, offset
            )
            .fetch_all(db)
            .await?
        }
        (Some(status), None) => {
            sqlx::query_as!(
                Case,
                "SELECT * FROM cases WHERE status = ?1 
                 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
                status, limit, offset
            )
            .fetch_all(db)
            .await?
        }
        (None, Some(assigned_to)) => {
            sqlx::query_as!(
                Case,
                "SELECT * FROM cases WHERE assigned_to = ?1 
                 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
                assigned_to, limit, offset
            )
            .fetch_all(db)
            .await?
        }
        (None, None) => {
            sqlx::query_as!(
                Case,
                "SELECT * FROM cases ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
                limit, offset
            )
            .fetch_all(db)
            .await?
        }
    };

    Ok(cases)
}

pub async fn update_case(
    db: &SqlitePool,
    id: &str,
    request: UpdateCaseRequest,
    updated_by: String,
) -> Result<Option<Case>> {
    let updated_at = Utc::now().to_rfc3339();

    // Get current case for comparison
    let current_case = match get_case(db, id).await? {
        Some(case) => case,
        None => return Ok(None),
    };

    let status_str = request.status.map(String::from);
    let priority_str = request.priority.map(String::from);

    // Update the case
    sqlx::query!(
        r#"
        UPDATE cases 
        SET title = COALESCE(?1, title),
            description = COALESCE(?2, description),
            status = COALESCE(?3, status),
            priority = COALESCE(?4, priority),
            assigned_to = COALESCE(?5, assigned_to),
            due_date = COALESCE(?6, due_date),
            updated_at = ?7
        WHERE id = ?8
        "#,
        request.title,
        request.description,
        status_str,
        priority_str,
        request.assigned_to,
        request.due_date,
        updated_at,
        id
    )
    .execute(db)
    .await?;

    // Create audit trail entries for changes
    let mut changes = Vec::new();
    
    if let Some(ref new_status) = request.status {
        let current_status = CaseStatus::from(current_case.status);
        if *new_status != current_status {
            changes.push(format!("Status changed from {:?} to {:?}", current_status, new_status));
        }
    }
    
    if let Some(ref new_assigned) = request.assigned_to {
        if current_case.assigned_to.as_ref() != Some(new_assigned) {
            changes.push(format!("Assigned to changed to {}", new_assigned));
        }
    }

    if !changes.is_empty() {
        let history = CaseHistory::new(
            id.to_string(),
            "Case updated".to_string(),
            Some(changes.join("; ")),
            updated_by,
        );

        sqlx::query!(
            "INSERT INTO case_history (id, case_id, action, details, performed_by, performed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            history.id,
            history.case_id,
            history.action,
            history.details,
            history.performed_by,
            history.performed_at
        )
        .execute(db)
        .await?;
    }

    info!("Updated case: {}", id);
    get_case(db, id).await
}

pub async fn get_case_documents(db: &SqlitePool, case_id: &str) -> Result<Vec<Document>> {
    let documents = sqlx::query_as!(
        Document,
        r#"
        SELECT d.* FROM documents d
        INNER JOIN case_documents cd ON d.id = cd.document_id
        WHERE cd.case_id = ?1
        ORDER BY cd.added_at DESC
        "#,
        case_id
    )
    .fetch_all(db)
    .await?;

    Ok(documents)
}

pub async fn add_document_to_case(
    db: &SqlitePool,
    case_id: &str,
    document_id: &str,
    added_by: String,
) -> Result<()> {
    let case_document = CaseDocument::new(
        case_id.to_string(),
        document_id.to_string(),
        added_by.clone(),
    );

    sqlx::query!(
        "INSERT INTO case_documents (id, case_id, document_id, added_by, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        case_document.id,
        case_document.case_id,
        case_document.document_id,
        case_document.added_by,
        case_document.added_at
    )
    .execute(db)
    .await?;

    // Create case history entry
    let history = CaseHistory::new(
        case_id.to_string(),
        "Document added".to_string(),
        Some(format!("Document {} added to case", document_id)),
        added_by,
    );

    sqlx::query!(
        "INSERT INTO case_history (id, case_id, action, details, performed_by, performed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        history.id,
        history.case_id,
        history.action,
        history.details,
        history.performed_by,
        history.performed_at
    )
    .execute(db)
    .await?;

    info!("Added document {} to case {}", document_id, case_id);
    Ok(())
}