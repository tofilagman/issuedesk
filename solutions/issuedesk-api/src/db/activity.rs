use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::Result, models::ActivityRow};

pub async fn list(pool: &PgPool, issue_id: Uuid) -> Result<Vec<ActivityRow>> {
    let rows = sqlx::query_as!(
        ActivityRow,
        r#"SELECT al.id, al.issue_id, al.actor_id, u.display_name as actor_name,
                  al.action, al.field, al.old_value, al.new_value, al.created_at
           FROM activity_log al
           JOIN users u ON u.id = al.actor_id
           WHERE al.issue_id = $1
           ORDER BY al.created_at DESC"#,
        issue_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
