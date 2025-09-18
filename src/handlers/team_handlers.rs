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

use crate::models::{CreateTeamRequest, UpdateTeamRequest, AddTeamMemberRequest, TeamRole};
use crate::services::team_service;

pub async fn create_team(
    State(db): State<SqlitePool>,
    Json(payload): Json<CreateTeamRequest>,
) -> Result<Json<Value>, StatusCode> {
    if let Err(err) = payload.validate() {
        error!("Team validation failed: {:?}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Extract user ID from JWT token
    let created_by = "system".to_string(); // Temporary placeholder

    match team_service::create_team(&db, payload, created_by).await {
        Ok(team) => Ok(Json(json!({
            "message": "Team created successfully",
            "team_id": team.id,
            "name": team.name
        }))),
        Err(err) => {
            error!("Failed to create team: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_team(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match team_service::get_team(&db, &id).await {
        Ok(Some(team)) => Ok(Json(serde_json::to_value(team).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(err) => {
            error!("Failed to get team: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_teams(
    State(db): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token
    let user_id = "system"; // Temporary placeholder

    let limit = params
        .get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(10);
    let offset = params
        .get("offset")
        .and_then(|o| o.parse().ok())
        .unwrap_or(0);

    match team_service::list_teams(&db, user_id, limit, offset).await {
        Ok(teams) => Ok(Json(json!({
            "teams": teams,
            "count": teams.len()
        }))),
        Err(err) => {
            error!("Failed to list teams: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_team(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTeamRequest>,
) -> Result<Json<Value>, StatusCode> {
    // This would be implemented with proper update logic
    Ok(Json(json!({
        "message": "Team update functionality not yet implemented",
        "team_id": id
    })))
}

pub async fn delete_team(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match team_service::delete_team(&db, &id).await {
        Ok(_) => Ok(Json(json!({
            "message": "Team deleted successfully",
            "team_id": id
        }))),
        Err(err) => {
            error!("Failed to delete team: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn add_team_member(
    State(db): State<SqlitePool>,
    Path(team_id): Path<String>,
    Json(payload): Json<AddTeamMemberRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Extract user ID from JWT token
    let added_by = "system".to_string(); // Temporary placeholder

    match team_service::add_team_member(&db, &team_id, payload, added_by).await {
        Ok(_) => Ok(Json(json!({
            "message": "Team member added successfully",
            "team_id": team_id
        }))),
        Err(err) => {
            error!("Failed to add team member: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn remove_team_member(
    State(db): State<SqlitePool>,
    Path((team_id, user_id)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    match team_service::remove_team_member(&db, &team_id, &user_id).await {
        Ok(_) => Ok(Json(json!({
            "message": "Team member removed successfully",
            "team_id": team_id,
            "user_id": user_id
        }))),
        Err(err) => {
            error!("Failed to remove team member: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_team_members(
    State(db): State<SqlitePool>,
    Path(team_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    match team_service::get_team_members(&db, &team_id).await {
        Ok(members) => Ok(Json(json!({
            "members": members,
            "count": members.len()
        }))),
        Err(err) => {
            error!("Failed to get team members: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_team_member_role(
    State(db): State<SqlitePool>,
    Path((team_id, user_id)): Path<(String, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Value>, StatusCode> {
    let role_str = payload.get("role")
        .and_then(|r| r.as_str())
        .unwrap_or("member");
    
    let role = TeamRole::from(role_str.to_string());

    match team_service::update_team_member_role(&db, &team_id, &user_id, role).await {
        Ok(_) => Ok(Json(json!({
            "message": "Team member role updated successfully",
            "team_id": team_id,
            "user_id": user_id
        }))),
        Err(err) => {
            error!("Failed to update team member role: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}