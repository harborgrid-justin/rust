use anyhow::Result;
use sqlx::SqlitePool;
use tracing::info;

use crate::models::{
    Team, TeamMember, TeamRole, CreateTeamRequest, AddTeamMemberRequest,
    TeamResponse, TeamMemberResponse, User
};

pub async fn create_team(
    db: &SqlitePool,
    request: CreateTeamRequest,
    created_by: String,
) -> Result<Team> {
    let team = Team::new(request.name, request.description, created_by.clone());

    sqlx::query!(
        r#"
        INSERT INTO teams (id, name, description, created_by, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        team.id,
        team.name,
        team.description,
        team.created_by,
        team.created_at,
        team.updated_at
    )
    .execute(db)
    .await?;

    // Add creator as team owner
    let owner_member = TeamMember::new(
        team.id.clone(),
        created_by.clone(),
        TeamRole::Owner,
        created_by
    );

    sqlx::query!(
        r#"
        INSERT INTO team_members (id, team_id, user_id, role, added_by, added_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        owner_member.id,
        owner_member.team_id,
        owner_member.user_id,
        owner_member.role,
        owner_member.added_by,
        owner_member.added_at
    )
    .execute(db)
    .await?;

    info!("Created team: {} ({})", team.name, team.id);
    Ok(team)
}

pub async fn get_team(db: &SqlitePool, id: &str) -> Result<Option<TeamResponse>> {
    let row = sqlx::query!(
        r#"
        SELECT t.id, t.name, t.description, t.created_by, t.created_at, t.updated_at,
               COUNT(tm.id) as "member_count: i32"
        FROM teams t
        LEFT JOIN team_members tm ON t.id = tm.team_id
        WHERE t.id = ?1
        GROUP BY t.id
        "#,
        id
    )
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(TeamResponse {
            id: row.id.unwrap_or_default(),
            name: row.name.unwrap_or_default(),
            description: row.description,
            member_count: row.member_count.unwrap_or(0) as i32,
            created_by: row.created_by.unwrap_or_default(),
            created_at: row.created_at.unwrap_or_default(),
            updated_at: row.updated_at.unwrap_or_default(),
        })),
        None => Ok(None),
    }
}

pub async fn list_teams(db: &SqlitePool, user_id: &str, limit: i32, offset: i32) -> Result<Vec<TeamResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT t.id, t.name, t.description, t.created_by, t.created_at, t.updated_at,
               COUNT(tm2.id) as "member_count: i32"
        FROM teams t
        INNER JOIN team_members tm ON t.id = tm.team_id AND tm.user_id = ?1
        LEFT JOIN team_members tm2 ON t.id = tm2.team_id
        GROUP BY t.id
        ORDER BY t.created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| TeamResponse {
            id: row.id.unwrap_or_default(),
            name: row.name.unwrap_or_default(),
            description: row.description,
            member_count: row.member_count.unwrap_or(0) as i32,
            created_by: row.created_by.unwrap_or_default(),
            created_at: row.created_at.unwrap_or_default(),
            updated_at: row.updated_at.unwrap_or_default(),
        })
        .collect())
}

pub async fn add_team_member(
    db: &SqlitePool,
    team_id: &str,
    request: AddTeamMemberRequest,
    added_by: String,
) -> Result<()> {
    let role = request.role.unwrap_or(TeamRole::Member);
    let member = TeamMember::new(team_id.to_string(), request.user_id, role, added_by);

    sqlx::query!(
        r#"
        INSERT INTO team_members (id, team_id, user_id, role, added_by, added_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        member.id,
        member.team_id,
        member.user_id,
        member.role,
        member.added_by,
        member.added_at
    )
    .execute(db)
    .await?;

    info!("Added user {} to team {}", member.user_id, team_id);
    Ok(())
}

pub async fn remove_team_member(
    db: &SqlitePool,
    team_id: &str,
    user_id: &str,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM team_members WHERE team_id = ?1 AND user_id = ?2",
        team_id,
        user_id
    )
    .execute(db)
    .await?;

    info!("Removed user {} from team {}", user_id, team_id);
    Ok(())
}

pub async fn get_team_members(db: &SqlitePool, team_id: &str) -> Result<Vec<TeamMemberResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT tm.id, tm.user_id, tm.role, tm.added_by, tm.added_at,
               u.username, u.full_name
        FROM team_members tm
        INNER JOIN users u ON tm.user_id = u.id
        WHERE tm.team_id = ?1
        ORDER BY tm.added_at ASC
        "#,
        team_id
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| TeamMemberResponse {
            id: row.id.unwrap_or_default(),
            user_id: row.user_id.unwrap_or_default(),
            username: row.username.unwrap_or_default(),
            full_name: row.full_name,
            role: TeamRole::from(row.role.unwrap_or_default()),
            added_by: row.added_by.unwrap_or_default(),
            added_at: row.added_at.unwrap_or_default(),
        })
        .collect())
}

pub async fn update_team_member_role(
    db: &SqlitePool,
    team_id: &str,
    user_id: &str,
    new_role: TeamRole,
) -> Result<()> {
    sqlx::query!(
        "UPDATE team_members SET role = ?1 WHERE team_id = ?2 AND user_id = ?3",
        String::from(new_role),
        team_id,
        user_id
    )
    .execute(db)
    .await?;

    info!("Updated user {} role in team {}", user_id, team_id);
    Ok(())
}

pub async fn delete_team(db: &SqlitePool, team_id: &str) -> Result<()> {
    // Team members will be deleted by CASCADE
    sqlx::query!("DELETE FROM teams WHERE id = ?1", team_id)
        .execute(db)
        .await?;

    info!("Deleted team {}", team_id);
    Ok(())
}