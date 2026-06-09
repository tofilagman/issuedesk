use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dto::IssueLink,
    error::{AppError, Result},
    models::enums::ActivityAction,
};

/// Map a user-facing link type (0-4, from the current issue's perspective) to the
/// canonical stored row `(source, target, type)`. Returns None for bad input.
fn to_canonical(facing: i16, current: Uuid, target: Uuid) -> Option<(Uuid, Uuid, i16)> {
    match facing {
        // relates: symmetric — order ids so the pair dedupes regardless of side
        0 => {
            let (a, b) = if current <= target { (current, target) } else { (target, current) };
            Some((a, b, 0))
        }
        1 => Some((current, target, 1)), // blocks
        2 => Some((target, current, 1)), // is blocked by  -> target blocks current
        3 => Some((current, target, 3)), // duplicates
        4 => Some((target, current, 3)), // is duplicated by -> target duplicates current
        _ => None,
    }
}

/// Derive the facing link type (0-4) for a stored row, given whether the viewing
/// issue is the stored `source`.
fn facing_of(canonical_type: i16, viewer_is_source: bool) -> i16 {
    match (canonical_type, viewer_is_source) {
        (1, true) => 1,  // blocks
        (1, false) => 2, // is blocked by
        (3, true) => 3,  // duplicates
        (3, false) => 4, // is duplicated by
        _ => 0,          // relates (symmetric)
    }
}

pub async fn list(pool: &PgPool, issue_id: Uuid) -> Result<Vec<IssueLink>> {
    let rows = sqlx::query!(
        r#"SELECT il.id, il.source_issue_id, il.target_issue_id, il.link_type,
                  s.number AS s_number, s.title AS s_title, s.status AS s_status, sp.key AS s_pkey,
                  t.number AS t_number, t.title AS t_title, t.status AS t_status, tp.key AS t_pkey
           FROM issue_links il
           JOIN issues   s  ON s.id  = il.source_issue_id
           JOIN projects sp ON sp.id = s.project_id
           JOIN issues   t  ON t.id  = il.target_issue_id
           JOIN projects tp ON tp.id = t.project_id
           WHERE il.source_issue_id = $1 OR il.target_issue_id = $1
           ORDER BY il.link_type, il.created_at"#,
        issue_id
    )
    .fetch_all(pool)
    .await?;

    let links = rows
        .into_iter()
        .map(|r| {
            let viewer_is_source = r.source_issue_id == issue_id;
            if viewer_is_source {
                IssueLink {
                    id: r.id,
                    link_type: facing_of(r.link_type, true),
                    issue_id: r.target_issue_id,
                    key: format!("{}-{}", r.t_pkey, r.t_number),
                    number: r.t_number,
                    title: r.t_title,
                    status: r.t_status,
                    project_key: r.t_pkey,
                }
            } else {
                IssueLink {
                    id: r.id,
                    link_type: facing_of(r.link_type, false),
                    issue_id: r.source_issue_id,
                    key: format!("{}-{}", r.s_pkey, r.s_number),
                    number: r.s_number,
                    title: r.s_title,
                    status: r.s_status,
                    project_key: r.s_pkey,
                }
            }
        })
        .collect();
    Ok(links)
}

pub async fn create(
    pool: &PgPool,
    current_issue: Uuid,
    target_issue: Uuid,
    facing: i16,
    actor_id: Uuid,
) -> Result<()> {
    let Some((s, t, ct)) = to_canonical(facing, current_issue, target_issue) else {
        return Err(AppError::BadRequest("invalid link type".into()));
    };

    let mut tx = pool.begin().await?;

    let inserted = sqlx::query!(
        r#"INSERT INTO issue_links (source_issue_id, target_issue_id, link_type, created_by)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (source_issue_id, target_issue_id, link_type) DO NOTHING
           RETURNING id"#,
        s,
        t,
        ct,
        actor_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    if inserted.is_none() {
        return Err(AppError::Conflict("these issues are already linked".into()));
    }

    // Record the link on both issues' activity feeds, each referencing the other.
    let cur_key = key_in_tx(&mut tx, current_issue).await?;
    let other_key = key_in_tx(&mut tx, target_issue).await?;
    log_link(&mut tx, current_issue, actor_id, ActivityAction::LinkAdded, &other_key).await?;
    log_link(&mut tx, target_issue, actor_id, ActivityAction::LinkAdded, &cur_key).await?;

    tx.commit().await?;
    Ok(())
}

/// Returns the two issue ids a link connects (for authorization).
pub async fn endpoints_of(pool: &PgPool, link_id: Uuid) -> Result<(Uuid, Uuid)> {
    let row = sqlx::query!(
        "SELECT source_issue_id, target_issue_id FROM issue_links WHERE id = $1",
        link_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("link not found".into()))?;
    Ok((row.source_issue_id, row.target_issue_id))
}

pub async fn delete(pool: &PgPool, link_id: Uuid, actor_id: Uuid) -> Result<()> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query!(
        "DELETE FROM issue_links WHERE id = $1 RETURNING source_issue_id, target_issue_id",
        link_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("link not found".into()))?;

    let s_key = key_in_tx(&mut tx, row.source_issue_id).await?;
    let t_key = key_in_tx(&mut tx, row.target_issue_id).await?;
    log_link(&mut tx, row.source_issue_id, actor_id, ActivityAction::LinkRemoved, &t_key).await?;
    log_link(&mut tx, row.target_issue_id, actor_id, ActivityAction::LinkRemoved, &s_key).await?;

    tx.commit().await?;
    Ok(())
}

// ----------------------------- helpers -----------------------------

async fn key_in_tx(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, issue_id: Uuid) -> Result<String> {
    let row = sqlx::query!(
        r#"SELECT p.key, i.number FROM issues i JOIN projects p ON p.id = i.project_id
           WHERE i.id = $1"#,
        issue_id
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(format!("{}-{}", row.key, row.number))
}

async fn log_link(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    issue_id: Uuid,
    actor_id: Uuid,
    action: ActivityAction,
    other_key: &str,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO activity_log (issue_id, actor_id, action, new_value) VALUES ($1, $2, $3, $4)",
        issue_id,
        actor_id,
        action.as_i16(),
        other_key
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
