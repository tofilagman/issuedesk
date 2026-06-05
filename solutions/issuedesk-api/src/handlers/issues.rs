use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    db,
    dto::{CreateIssueRequest, IssueDetail, IssueFilter, IssueListResponse, UpdateIssueRequest},
    error::Result,
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
    Query(filter): Query<IssueFilter>,
) -> Result<Json<IssueListResponse>> {
    db::authorize_project(&state.pool, &user, project_id).await?;
    let project = db::projects::get(&state.pool, project_id).await?;
    let resp = db::issues::list(&state.pool, project_id, &project.key, &filter).await?;
    Ok(Json(resp))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
    Json(req): Json<CreateIssueRequest>,
) -> Result<Json<IssueDetail>> {
    db::authorize_project(&state.pool, &user, project_id).await?;
    req.validate()?;
    let id = db::issues::create(&state.pool, project_id, user.id(), &req).await?;
    let detail = db::issues::get_detail(&state.pool, id).await?;
    Ok(Json(detail))
}

pub async fn get_by_number(
    State(state): State<AppState>,
    user: AuthUser,
    Path((project_id, number)): Path<(Uuid, i64)>,
) -> Result<Json<IssueDetail>> {
    db::authorize_project(&state.pool, &user, project_id).await?;
    let id = db::issues::id_by_number(&state.pool, project_id, number).await?;
    let detail = db::issues::get_detail(&state.pool, id).await?;
    Ok(Json(detail))
}

pub async fn get(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
) -> Result<Json<IssueDetail>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    let detail = db::issues::get_detail(&state.pool, issue_id).await?;
    Ok(Json(detail))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
    Json(req): Json<UpdateIssueRequest>,
) -> Result<Json<IssueDetail>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    req.validate()?;
    let detail = db::issues::update(&state.pool, issue_id, user.id(), &req).await?;
    Ok(Json(detail))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    db::issues::delete(&state.pool, issue_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
