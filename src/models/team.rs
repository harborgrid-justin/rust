use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamMember {
    pub id: String,
    pub team_id: String,
    pub user_id: String,
    pub role: String,
    pub added_by: String,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeamRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl From<String> for TeamRole {
    fn from(s: String) -> Self {
        match s.as_str() {
            "owner" => TeamRole::Owner,
            "admin" => TeamRole::Admin,
            "member" => TeamRole::Member,
            "viewer" => TeamRole::Viewer,
            _ => TeamRole::Member,
        }
    }
}

impl From<TeamRole> for String {
    fn from(role: TeamRole) -> Self {
        match role {
            TeamRole::Owner => "owner".to_string(),
            TeamRole::Admin => "admin".to_string(),
            TeamRole::Member => "member".to_string(),
            TeamRole::Viewer => "viewer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTeamRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddTeamMemberRequest {
    pub user_id: String,
    pub role: Option<TeamRole>,
}

#[derive(Debug, Serialize)]
pub struct TeamResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i32,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct TeamMemberResponse {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub full_name: Option<String>,
    pub role: TeamRole,
    pub added_by: String,
    pub added_at: String,
}

impl Team {
    pub fn new(name: String, description: Option<String>, created_by: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_by,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl TeamMember {
    pub fn new(team_id: String, user_id: String, role: TeamRole, added_by: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            team_id,
            user_id,
            role: String::from(role),
            added_by,
            added_at: Utc::now().to_rfc3339(),
        }
    }
}