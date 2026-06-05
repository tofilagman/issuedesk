use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    db,
    dto::{AttachLabelRequest, CreateLabelRequest, COLOR_RE},
    error::{AppError, Result},
    models::LabelRow,
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<LabelRow>>> {
    db::authorize_project(&state.pool, &user, project_id).await?;
    let rows = db::labels::list(&state.pool, project_id).await?;
    Ok(Json(rows))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateLabelRequest>,
) -> Result<Json<LabelRow>> {
    db::authorize_project(&state.pool, &user, project_id).await?;
    req.validate()?;
    if !COLOR_RE.is_match(&req.color) {
        return Err(AppError::Validation("color must be a #RRGGBB hex value".into()));
    }
    let row = db::labels::create(&state.pool, project_id, &req.name, &req.color).await?;
    Ok(Json(row))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(label_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let project_id = db::labels::project_of(&state.pool, label_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    db::labels::delete(&state.pool, label_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn attach(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
    Json(req): Json<AttachLabelRequest>,
) -> Result<Json<serde_json::Value>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    // Ensure the label belongs to the same project.
    let label_project = db::labels::project_of(&state.pool, req.label_id).await?;
    if label_project != project_id {
        return Err(AppError::BadRequest("label belongs to a different project".into()));
    }
    db::labels::attach(&state.pool, issue_id, req.label_id, user.id()).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn detach(
    State(state): State<AppState>,
    user: AuthUser,
    Path((issue_id, label_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    db::labels::detach(&state.pool, issue_id, label_id, user.id()).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
