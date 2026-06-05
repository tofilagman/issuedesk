use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{auth::AuthUser, db, error::Result, models::ActivityRow, state::AppState};

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
) -> Result<Json<Vec<ActivityRow>>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    let rows = db::activity::list(&state.pool, issue_id).await?;
    Ok(Json(rows))
}
