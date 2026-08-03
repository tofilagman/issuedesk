//! Admin-managed user groups. Customers only see issues reported by
//! themselves or by users sharing at least one group with them.

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    db,
    dto::{AddGroupMemberRequest, CreateGroupRequest, UpdateGroupRequest},
    error::Result,
    models::{GroupRow, MemberRow},
    state::AppState,
};

pub async fn list(State(state): State<AppState>, user: AuthUser) -> Result<Json<Vec<GroupRow>>> {
    user.require_admin()?;
    let rows = db::groups::list(&state.pool).await?;
    Ok(Json(rows))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateGroupRequest>,
) -> Result<Json<GroupRow>> {
    user.require_admin()?;
    req.validate()?;
    let row = db::groups::create(&state.pool, &req.name, req.description.as_deref(), user.id()).await?;
    Ok(Json(row))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(group_id): Path<Uuid>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<GroupRow>> {
    user.require_admin()?;
    req.validate()?;
    let row = db::groups::update(&state.pool, group_id, req.name.as_deref(), req.description.as_deref()).await?;
    Ok(Json(row))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(group_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    user.require_admin()?;
    db::groups::delete(&state.pool, group_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn members(
    State(state): State<AppState>,
    user: AuthUser,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Vec<MemberRow>>> {
    user.require_admin()?;
    let rows = db::groups::list_members(&state.pool, group_id).await?;
    Ok(Json(rows))
}

pub async fn add_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(group_id): Path<Uuid>,
    Json(req): Json<AddGroupMemberRequest>,
) -> Result<Json<Vec<MemberRow>>> {
    user.require_admin()?;
    db::groups::add_member(&state.pool, group_id, req.user_id).await?;
    let rows = db::groups::list_members(&state.pool, group_id).await?;
    Ok(Json(rows))
}

pub async fn remove_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>> {
    user.require_admin()?;
    db::groups::remove_member(&state.pool, group_id, user_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
