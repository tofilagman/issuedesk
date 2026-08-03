use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    db,
    dto::{CreateIssueRequest, IssueDetail, IssueFilter, IssueListResponse, ReorderRequest, UpdateIssueRequest},
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
    // Customers only see their own tickets and their group-mates'.
    let visible_to = user.is_customer().then(|| user.id());
    let resp = db::issues::list(&state.pool, project_id, &project.key, &filter, visible_to).await?;
    Ok(Json(resp))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
    Json(mut req): Json<CreateIssueRequest>,
) -> Result<Json<IssueDetail>> {
    db::authorize_project(&state.pool, &user, project_id).await?;
    req.validate()?;
    // Customers report tickets but don't triage them.
    if user.is_customer() {
        req.assignee_id = None;
        req.label_ids = None;
    }
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
    if user.is_customer() && !db::issues::visible_to_user(&state.pool, id, user.id()).await? {
        return Err(crate::error::AppError::NotFound("issue not found".into()));
    }
    let detail = db::issues::get_detail(&state.pool, id).await?;
    Ok(Json(detail))
}

pub async fn get(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
) -> Result<Json<IssueDetail>> {
    db::authorize_issue(&state.pool, &user, issue_id).await?;
    let detail = db::issues::get_detail(&state.pool, issue_id).await?;
    Ok(Json(detail))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
    Json(req): Json<UpdateIssueRequest>,
) -> Result<Json<IssueDetail>> {
    db::authorize_issue(&state.pool, &user, issue_id).await?;
    if user.is_customer() {
        let (_, reporter_id) = db::issues::project_and_reporter(&state.pool, issue_id).await?;
        if reporter_id != user.id() {
            return Err(crate::error::AppError::Forbidden(
                "customers may only edit issues they reported".into(),
            ));
        }
        // assignee_id.is_some() also catches an explicit null (unassign).
        if req.r#type.is_some() || req.status.is_some() || req.priority.is_some() || req.assignee_id.is_some() {
            return Err(crate::error::AppError::Forbidden(
                "customers may only edit title and description".into(),
            ));
        }
    }
    req.validate()?;
    let detail = db::issues::update(&state.pool, issue_id, user.id(), &req).await?;
    Ok(Json(detail))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    user.require_not_customer()?;
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    db::issues::delete(&state.pool, issue_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// `PATCH /api/issues/{id}/position` — move an issue within/into a board column.
pub async fn reorder(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
    Json(req): Json<ReorderRequest>,
) -> Result<Json<serde_json::Value>> {
    user.require_not_customer()?;
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    req.validate()?;
    db::issues::reorder(&state.pool, issue_id, user.id(), &req).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
