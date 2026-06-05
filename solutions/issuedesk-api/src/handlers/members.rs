use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    dto::AddMemberRequest,
    error::Result,
    models::MemberRow,
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<MemberRow>>> {
    db::authorize_project(&state.pool, &user, project_id).await?;
    let rows = db::projects::list_members(&state.pool, project_id).await?;
    Ok(Json(rows))
}

pub async fn add(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> Result<Json<Vec<MemberRow>>> {
    // Only admins or project members can manage membership.
    db::authorize_project(&state.pool, &user, project_id).await?;
    db::projects::add_member(&state.pool, project_id, req.user_id, req.role.unwrap_or(0)).await?;
    let rows = db::projects::list_members(&state.pool, project_id).await?;
    Ok(Json(rows))
}

pub async fn remove(
    State(state): State<AppState>,
    user: AuthUser,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>> {
    db::authorize_project(&state.pool, &user, project_id).await?;
    db::projects::remove_member(&state.pool, project_id, user_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
