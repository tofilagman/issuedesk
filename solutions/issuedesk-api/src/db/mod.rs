pub mod activity;
pub mod api_keys;
pub mod attachments;
pub mod comments;
pub mod groups;
pub mod issues;
pub mod labels;
pub mod links;
pub mod projects;
pub mod stats;
pub mod users;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::AuthUser, error::Result};

/// Authorize a project-scoped action: admins always pass; otherwise the caller
/// must be a member of the project.
pub async fn authorize_project(pool: &PgPool, user: &AuthUser, project_id: Uuid) -> Result<()> {
    if user.is_admin() {
        return Ok(());
    }
    if projects::is_member(pool, project_id, user.id()).await? {
        Ok(())
    } else {
        Err(crate::error::AppError::Forbidden(
            "not a member of this project".into(),
        ))
    }
}

/// Authorize an issue-scoped action and return the issue's project id. Admins
/// pass; members must belong to the project; customers must additionally
/// satisfy reporter-visibility (own issue, or reporter shares a group). Fails
/// as NotFound for customers so they cannot probe issue existence.
pub async fn authorize_issue(pool: &PgPool, user: &AuthUser, issue_id: Uuid) -> Result<Uuid> {
    let project_id = issues::project_of(pool, issue_id).await?;
    authorize_project(pool, user, project_id).await?;
    if user.is_customer() && !issues::visible_to_user(pool, issue_id, user.id()).await? {
        return Err(crate::error::AppError::NotFound("issue not found".into()));
    }
    Ok(project_id)
}
