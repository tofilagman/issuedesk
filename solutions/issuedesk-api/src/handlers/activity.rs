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
    db::authorize_issue(&state.pool, &user, issue_id).await?;
    let rows = db::activity::list(&state.pool, issue_id).await?;
    Ok(Json(rows))
}
