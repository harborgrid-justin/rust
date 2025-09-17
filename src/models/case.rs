use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Case {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String, // Changed to String for SQLite compatibility
    pub priority: String, // Changed to String for SQLite compatibility
    pub assigned_to: Option<String>,
    pub created_by: String,
    pub due_date: Option<String>, // Changed to String for SQLite compatibility
    pub closed_at: Option<String>, // Changed to String for SQLite compatibility
    pub created_at: String, // Changed to String for SQLite compatibility
    pub updated_at: String, // Changed to String for SQLite compatibility
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaseStatus {
    Open,
    InProgress,
    UnderReview,
    Resolved,
    Closed,
}

impl From<String> for CaseStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "open" => CaseStatus::Open,
            "inprogress" => CaseStatus::InProgress,
            "underreview" => CaseStatus::UnderReview,
            "resolved" => CaseStatus::Resolved,
            "closed" => CaseStatus::Closed,
            _ => CaseStatus::Open,
        }
    }
}

impl From<CaseStatus> for String {
    fn from(status: CaseStatus) -> Self {
        match status {
            CaseStatus::Open => "open".to_string(),
            CaseStatus::InProgress => "inprogress".to_string(),
            CaseStatus::UnderReview => "underreview".to_string(),
            CaseStatus::Resolved => "resolved".to_string(),
            CaseStatus::Closed => "closed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CasePriority {
    Low,
    Medium,
    High,
    Critical,
}

impl From<String> for CasePriority {
    fn from(s: String) -> Self {
        match s.as_str() {
            "low" => CasePriority::Low,
            "medium" => CasePriority::Medium,
            "high" => CasePriority::High,
            "critical" => CasePriority::Critical,
            _ => CasePriority::Medium,
        }
    }
}

impl From<CasePriority> for String {
    fn from(priority: CasePriority) -> Self {
        match priority {
            CasePriority::Low => "low".to_string(),
            CasePriority::Medium => "medium".to_string(),
            CasePriority::High => "high".to_string(),
            CasePriority::Critical => "critical".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CaseDocument {
    pub id: String,
    pub case_id: String,
    pub document_id: String,
    pub added_by: String,
    pub added_at: String, // Changed to String for SQLite compatibility
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CaseHistory {
    pub id: String,
    pub case_id: String,
    pub action: String,
    pub details: Option<String>,
    pub performed_by: String,
    pub performed_at: String, // Changed to String for SQLite compatibility
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCaseRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    
    pub description: Option<String>,
    pub priority: Option<CasePriority>,
    pub assigned_to: Option<String>,
    pub due_date: Option<String>, // Accept as string for easier handling
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCaseRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<CaseStatus>,
    pub priority: Option<CasePriority>,
    pub assigned_to: Option<String>,
    pub due_date: Option<String>, // Accept as string for easier handling
}

#[derive(Debug, Serialize)]
pub struct CaseResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: CaseStatus,
    pub priority: CasePriority,
    pub assigned_to: Option<String>,
    pub created_by: String,
    pub due_date: Option<String>,
    pub closed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Case> for CaseResponse {
    fn from(case: Case) -> Self {
        Self {
            id: case.id,
            title: case.title,
            description: case.description,
            status: CaseStatus::from(case.status),
            priority: CasePriority::from(case.priority),
            assigned_to: case.assigned_to,
            created_by: case.created_by,
            due_date: case.due_date,
            closed_at: case.closed_at,
            created_at: case.created_at,
            updated_at: case.updated_at,
        }
    }
}

impl Case {
    pub fn new(
        title: String,
        description: Option<String>,
        priority: CasePriority,
        assigned_to: Option<String>,
        created_by: String,
        due_date: Option<String>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();

        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            status: String::from(CaseStatus::Open),
            priority: String::from(priority),
            assigned_to,
            created_by,
            due_date,
            closed_at: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl CaseDocument {
    pub fn new(case_id: String, document_id: String, added_by: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            case_id,
            document_id,
            added_by,
            added_at: Utc::now().to_rfc3339(),
        }
    }
}

impl CaseHistory {
    pub fn new(case_id: String, action: String, details: Option<String>, performed_by: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            case_id,
            action,
            details,
            performed_by,
            performed_at: Utc::now().to_rfc3339(),
        }
    }
}