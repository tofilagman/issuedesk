use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    dto::{ProjectStats, SystemStats},
    error::Result,
    state::AppState,
};

pub async fn project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectStats>> {
    db::authorize_project(&state.pool, &user, project_id).await?;
    let stats = db::stats::project_stats(&state.pool, project_id).await?;
    Ok(Json(stats))
}

pub async fn system(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<SystemStats>> {
    // Scope to the projects this user can see (admins see all).
    let projects = db::projects::list_visible(&state.pool, user.id(), user.is_admin()).await?;
    let ids: Vec<Uuid> = projects.iter().map(|p| p.id).collect();
    let stats = db::stats::system_stats(&state.pool, &ids).await?;
    Ok(Json(stats))
}
