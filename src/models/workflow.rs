use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CaseTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_priority: String,
    pub default_assignee: Option<String>,
    pub workflow_steps: Option<String>, // JSON array
    pub custom_fields: Option<String>, // JSON object
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CaseWorkflow {
    pub id: String,
    pub case_id: String,
    pub step_name: String,
    pub step_order: i32,
    pub status: String,
    pub assigned_to: Option<String>,
    pub completed_by: Option<String>,
    pub completed_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CaseCustomField {
    pub id: String,
    pub case_id: String,
    pub field_name: String,
    pub field_type: String,
    pub field_value: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

impl From<String> for WorkflowStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => WorkflowStatus::Pending,
            "in_progress" => WorkflowStatus::InProgress,
            "completed" => WorkflowStatus::Completed,
            "skipped" => WorkflowStatus::Skipped,
            _ => WorkflowStatus::Pending,
        }
    }
}

impl From<WorkflowStatus> for String {
    fn from(status: WorkflowStatus) -> Self {
        match status {
            WorkflowStatus::Pending => "pending".to_string(),
            WorkflowStatus::InProgress => "in_progress".to_string(),
            WorkflowStatus::Completed => "completed".to_string(),
            WorkflowStatus::Skipped => "skipped".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CustomFieldType {
    Text,
    Number,
    Date,
    Boolean,
    Select,
}

impl From<String> for CustomFieldType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "text" => CustomFieldType::Text,
            "number" => CustomFieldType::Number,
            "date" => CustomFieldType::Date,
            "boolean" => CustomFieldType::Boolean,
            "select" => CustomFieldType::Select,
            _ => CustomFieldType::Text,
        }
    }
}

impl From<CustomFieldType> for String {
    fn from(field_type: CustomFieldType) -> Self {
        match field_type {
            CustomFieldType::Text => "text".to_string(),
            CustomFieldType::Number => "number".to_string(),
            CustomFieldType::Date => "date".to_string(),
            CustomFieldType::Boolean => "boolean".to_string(),
            CustomFieldType::Select => "select".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub order: i32,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomFieldSchema {
    pub name: String,
    pub field_type: CustomFieldType,
    pub required: bool,
    pub default_value: Option<String>,
    pub options: Option<Vec<String>>, // For select type
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCaseTemplateRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    pub default_priority: Option<String>,
    pub default_assignee: Option<String>,
    pub workflow_steps: Option<Vec<WorkflowStep>>,
    pub custom_fields: Option<Vec<CustomFieldSchema>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowStepRequest {
    pub status: Option<WorkflowStatus>,
    pub assigned_to: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetCustomFieldRequest {
    pub field_value: String,
}

#[derive(Debug, Serialize)]
pub struct CaseTemplateResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_priority: String,
    pub default_assignee: Option<String>,
    pub workflow_steps: Vec<WorkflowStep>,
    pub custom_fields: Vec<CustomFieldSchema>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<CaseTemplate> for CaseTemplateResponse {
    fn from(template: CaseTemplate) -> Self {
        let workflow_steps = template.workflow_steps
            .as_ref()
            .and_then(|s| serde_json::from_str::<Vec<WorkflowStep>>(s).ok())
            .unwrap_or_default();

        let custom_fields = template.custom_fields
            .as_ref()
            .and_then(|s| serde_json::from_str::<Vec<CustomFieldSchema>>(s).ok())
            .unwrap_or_default();

        Self {
            id: template.id,
            name: template.name,
            description: template.description,
            default_priority: template.default_priority,
            default_assignee: template.default_assignee,
            workflow_steps,
            custom_fields,
            created_by: template.created_by,
            created_at: template.created_at,
            updated_at: template.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CaseWorkflowResponse {
    pub id: String,
    pub case_id: String,
    pub step_name: String,
    pub step_order: i32,
    pub status: WorkflowStatus,
    pub assigned_to: Option<String>,
    pub completed_by: Option<String>,
    pub completed_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<CaseWorkflow> for CaseWorkflowResponse {
    fn from(workflow: CaseWorkflow) -> Self {
        Self {
            id: workflow.id,
            case_id: workflow.case_id,
            step_name: workflow.step_name,
            step_order: workflow.step_order,
            status: WorkflowStatus::from(workflow.status),
            assigned_to: workflow.assigned_to,
            completed_by: workflow.completed_by,
            completed_at: workflow.completed_at,
            notes: workflow.notes,
            created_at: workflow.created_at,
            updated_at: workflow.updated_at,
        }
    }
}

impl CaseTemplate {
    pub fn new(
        name: String,
        description: Option<String>,
        default_priority: String,
        default_assignee: Option<String>,
        workflow_steps: Option<Vec<WorkflowStep>>,
        custom_fields: Option<Vec<CustomFieldSchema>>,
        created_by: String,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        let workflow_json = workflow_steps.map(|steps| serde_json::to_string(&steps).unwrap_or_default());
        let fields_json = custom_fields.map(|fields| serde_json::to_string(&fields).unwrap_or_default());

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            default_priority,
            default_assignee,
            workflow_steps: workflow_json,
            custom_fields: fields_json,
            created_by,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl CaseWorkflow {
    pub fn new(
        case_id: String,
        step_name: String,
        step_order: i32,
        assigned_to: Option<String>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        
        Self {
            id: Uuid::new_v4().to_string(),
            case_id,
            step_name,
            step_order,
            status: String::from(WorkflowStatus::Pending),
            assigned_to,
            completed_by: None,
            completed_at: None,
            notes: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl CaseCustomField {
    pub fn new(
        case_id: String,
        field_name: String,
        field_type: CustomFieldType,
        field_value: Option<String>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        
        Self {
            id: Uuid::new_v4().to_string(),
            case_id,
            field_name,
            field_type: String::from(field_type),
            field_value,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}