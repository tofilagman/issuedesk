use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    db,
    dto::{CreateLinkRequest, IssueLink},
    error::{AppError, Result},
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
) -> Result<Json<Vec<IssueLink>>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    let rows = db::links::list(&state.pool, issue_id).await?;
    Ok(Json(rows))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
    Json(req): Json<CreateLinkRequest>,
) -> Result<Json<Vec<IssueLink>>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    req.validate()?;

    if req.target_issue_id == issue_id {
        return Err(AppError::BadRequest("cannot link an issue to itself".into()));
    }
    // MVP: links are within a single project (keeps authorization + search simple).
    let target_project = db::issues::project_of(&state.pool, req.target_issue_id).await?;
    if target_project != project_id {
        return Err(AppError::BadRequest(
            "can only link issues in the same project".into(),
        ));
    }

    db::links::create(&state.pool, issue_id, req.target_issue_id, req.link_type, user.id()).await?;
    let rows = db::links::list(&state.pool, issue_id).await?;
    Ok(Json(rows))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(link_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let (source, _target) = db::links::endpoints_of(&state.pool, link_id).await?;
    let project_id = db::issues::project_of(&state.pool, source).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    db::links::delete(&state.pool, link_id, user.id()).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
