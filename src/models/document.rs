use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub status: String, // Changed to String for SQLite compatibility
    pub version: i32,
    pub tags: Option<String>, // JSON string of tags
    pub created_by: String,
    pub created_at: String, // Changed to String for SQLite compatibility
    pub updated_at: String, // Changed to String for SQLite compatibility
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentStatus {
    Draft,
    Review,
    Approved,
    Published,
    Archived,
}

impl From<String> for DocumentStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "draft" => DocumentStatus::Draft,
            "review" => DocumentStatus::Review,
            "approved" => DocumentStatus::Approved,
            "published" => DocumentStatus::Published,
            "archived" => DocumentStatus::Archived,
            _ => DocumentStatus::Draft,
        }
    }
}

impl From<DocumentStatus> for String {
    fn from(status: DocumentStatus) -> Self {
        match status {
            DocumentStatus::Draft => "draft".to_string(),
            DocumentStatus::Review => "review".to_string(),
            DocumentStatus::Approved => "approved".to_string(),
            DocumentStatus::Published => "published".to_string(),
            DocumentStatus::Archived => "archived".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDocumentRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<DocumentStatus>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub status: DocumentStatus,
    pub version: i32,
    pub tags: Vec<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Document> for DocumentResponse {
    fn from(doc: Document) -> Self {
        let tags = doc.tags
            .as_ref()
            .and_then(|t| serde_json::from_str::<Vec<String>>(t).ok())
            .unwrap_or_default();

        Self {
            id: doc.id,
            title: doc.title,
            description: doc.description,
            file_name: doc.file_name,
            file_size: doc.file_size,
            mime_type: doc.mime_type,
            status: DocumentStatus::from(doc.status),
            version: doc.version,
            tags,
            created_by: doc.created_by,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

impl Document {
    pub fn new(title: String, description: Option<String>, created_by: String, tags: Option<Vec<String>>) -> Self {
        let now = Utc::now().to_rfc3339();
        let tags_json = tags.map(|t| serde_json::to_string(&t).unwrap_or_default());

        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            file_path: None,
            file_name: None,
            file_size: None,
            mime_type: None,
            status: String::from(DocumentStatus::Draft),
            version: 1,
            tags: tags_json,
            created_by,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}