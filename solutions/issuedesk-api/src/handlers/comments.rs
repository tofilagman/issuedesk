use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    db,
    dto::{CreateCommentRequest, UpdateCommentRequest},
    error::{AppError, Result},
    models::CommentRow,
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
) -> Result<Json<Vec<CommentRow>>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    let rows = db::comments::list(&state.pool, issue_id).await?;
    Ok(Json(rows))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<CommentRow>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    req.validate()?;
    let row = db::comments::create(&state.pool, issue_id, user.id(), &req.body).await?;
    Ok(Json(row))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(comment_id): Path<Uuid>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<Json<CommentRow>> {
    let author = db::comments::author_of(&state.pool, comment_id).await?;
    if !user.is_admin() && user.id() != author {
        return Err(AppError::Forbidden("not your comment".into()));
    }
    req.validate()?;
    let row = db::comments::update(&state.pool, comment_id, &req.body).await?;
    Ok(Json(row))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(comment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let author = db::comments::author_of(&state.pool, comment_id).await?;
    if !user.is_admin() && user.id() != author {
        return Err(AppError::Forbidden("not your comment".into()));
    }
    db::comments::delete(&state.pool, comment_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
