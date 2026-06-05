use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::{password, AuthUser},
    db,
    dto::{ChangePasswordRequest, CreateUserRequest, UpdateUserRequest},
    error::{AppError, Result},
    models::UserDto,
    state::AppState,
};

pub async fn me(State(state): State<AppState>, user: AuthUser) -> Result<Json<UserDto>> {
    let row = db::users::find_by_id(&state.pool, user.id())
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    Ok(Json(UserDto::from(row)))
}

pub async fn list(State(state): State<AppState>, user: AuthUser) -> Result<Json<Vec<UserDto>>> {
    user.require_admin()?;
    let rows = db::users::list(&state.pool).await?;
    Ok(Json(rows.into_iter().map(UserDto::from).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserDto>> {
    user.require_admin()?;
    req.validate()?;
    let hash = password::hash_password(&req.password)?;
    let row = db::users::create(
        &state.pool,
        &req.user_name,
        &req.email,
        &req.display_name,
        &hash,
        req.role.unwrap_or(0),
    )
    .await?;
    Ok(Json(UserDto::from(row)))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserDto>> {
    user.require_admin()?;
    req.validate()?;
    let row = db::users::update(
        &state.pool,
        id,
        req.display_name.as_deref(),
        req.email.as_deref(),
        req.role,
        req.is_active,
    )
    .await?;
    Ok(Json(UserDto::from(row)))
}

pub async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>> {
    // Admins can reset anyone; a user can change their own password.
    if !user.is_admin() && user.id() != id {
        return Err(AppError::Forbidden("cannot change another user's password".into()));
    }
    req.validate()?;
    let hash = password::hash_password(&req.password)?;
    db::users::set_password(&state.pool, id, &hash).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
