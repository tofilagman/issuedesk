use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    db,
    dto::{CreateProjectRequest, UpdateProjectRequest, PROJECT_KEY_RE},
    error::{AppError, Result},
    models::ProjectRow,
    state::AppState,
};

pub async fn list(State(state): State<AppState>, user: AuthUser) -> Result<Json<Vec<ProjectRow>>> {
    let rows = db::projects::list_visible(&state.pool, user.id(), user.is_admin()).await?;
    Ok(Json(rows))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<ProjectRow>> {
    user.require_admin()?;
    req.validate()?;
    let key = req.key.to_uppercase();
    if !PROJECT_KEY_RE.is_match(&key) {
        return Err(AppError::Validation(
            "key must be 2-10 chars, uppercase letters/digits, starting with a letter".into(),
        ));
    }
    let row = db::projects::create(
        &state.pool,
        &key,
        &req.name,
        req.description.as_deref(),
        user.id(),
    )
    .await?;
    Ok(Json(row))
}

pub async fn get(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectRow>> {
    db::authorize_project(&state.pool, &user, id).await?;
    let row = db::projects::get(&state.pool, id).await?;
    Ok(Json(row))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectRow>> {
    db::authorize_project(&state.pool, &user, id).await?;
    req.validate()?;
    let row = db::projects::update(&state.pool, id, req.name.as_deref(), req.description.as_deref())
        .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    user.require_admin()?;
    db::projects::delete(&state.pool, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
