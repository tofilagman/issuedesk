pub mod activity;
pub mod attachments;
pub mod comments;
pub mod issues;
pub mod labels;
pub mod projects;
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
