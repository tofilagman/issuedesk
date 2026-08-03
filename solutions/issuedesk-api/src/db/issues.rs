use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dto::{CreateIssueRequest, IssueDetail, IssueFilter, IssueListItem, IssueListResponse, ReorderRequest, UpdateIssueRequest},
    error::{AppError, Result},
    models::{enums::ActivityAction, LabelRow},
};

/// Create an issue, assigning the next per-project number atomically.
pub async fn create(
    pool: &PgPool,
    project_id: Uuid,
    reporter_id: Uuid,
    req: &CreateIssueRequest,
) -> Result<Uuid> {
    let mut tx = pool.begin().await?;

    // Row-lock the project row and bump its counter; serializes per project.
    let seq = sqlx::query!(
        "UPDATE projects SET issue_seq = issue_seq + 1, updated_at = now()
         WHERE id = $1 RETURNING issue_seq",
        project_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("project not found".into()))?
    .issue_seq;

    let r#type = req.r#type.unwrap_or(1);
    let priority = req.priority.unwrap_or(1);

    // New issues land at the top of the Todo column (status 0): one step above
    // whatever is currently first there.
    let board_position = sqlx::query_scalar!(
        "SELECT COALESCE(MIN(board_position), 1) - 1 FROM issues WHERE project_id = $1 AND status = 0",
        project_id
    )
    .fetch_one(&mut *tx)
    .await?
    .unwrap_or(0.0);

    let issue_id = sqlx::query!(
        r#"INSERT INTO issues
              (project_id, number, title, description, type, status, priority, assignee_id, reporter_id, board_position)
           VALUES ($1, $2, $3, $4, $5, 0, $6, $7, $8, $9)
           RETURNING id"#,
        project_id,
        seq,
        req.title,
        req.description.as_deref(),
        r#type,
        priority,
        req.assignee_id,
        reporter_id,
        board_position
    )
    .fetch_one(&mut *tx)
    .await?
    .id;

    if let Some(label_ids) = &req.label_ids {
        for lid in label_ids {
            sqlx::query!(
                "INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)
                 ON CONFLICT DO NOTHING",
                issue_id,
                lid
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    sqlx::query!(
        "INSERT INTO activity_log (issue_id, actor_id, action) VALUES ($1, $2, $3)",
        issue_id,
        reporter_id,
        ActivityAction::Created.as_i16()
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(issue_id)
}

/// Full issue detail (with project key, assignee/reporter names, labels).
pub async fn get_detail(pool: &PgPool, issue_id: Uuid) -> Result<IssueDetail> {
    let row = sqlx::query!(
        r#"SELECT i.id, i.project_id, p.key as project_key, i.number, i.title, i.description,
                  i.type, i.status, i.priority,
                  i.assignee_id, a.display_name as "assignee_name?",
                  i.reporter_id, r.display_name as reporter_name,
                  i.created_at, i.updated_at
           FROM issues i
           JOIN projects p ON p.id = i.project_id
           JOIN users r ON r.id = i.reporter_id
           LEFT JOIN users a ON a.id = i.assignee_id
           WHERE i.id = $1"#,
        issue_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("issue not found".into()))?;

    let labels = labels_for_issue(pool, issue_id).await?;

    Ok(IssueDetail {
        id: row.id,
        project_id: row.project_id,
        key: format!("{}-{}", row.project_key, row.number),
        project_key: row.project_key,
        number: row.number,
        title: row.title,
        description: row.description,
        r#type: row.r#type,
        status: row.status,
        priority: row.priority,
        assignee_id: row.assignee_id,
        assignee_name: row.assignee_name,
        reporter_id: row.reporter_id,
        reporter_name: row.reporter_name,
        created_at: row.created_at,
        updated_at: row.updated_at,
        labels,
    })
}

/// Resolve an issue id from its project + per-project number.
pub async fn id_by_number(pool: &PgPool, project_id: Uuid, number: i64) -> Result<Uuid> {
    let row = sqlx::query!(
        "SELECT id FROM issues WHERE project_id = $1 AND number = $2",
        project_id,
        number
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("issue not found".into()))?;
    Ok(row.id)
}

/// Resolve an issue's project id (for authorization) without loading everything.
pub async fn project_of(pool: &PgPool, issue_id: Uuid) -> Result<Uuid> {
    let row = sqlx::query!("SELECT project_id FROM issues WHERE id = $1", issue_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("issue not found".into()))?;
    Ok(row.project_id)
}

/// Project and reporter in one round trip (for customer ownership checks).
pub async fn project_and_reporter(pool: &PgPool, issue_id: Uuid) -> Result<(Uuid, Uuid)> {
    let row = sqlx::query!(
        "SELECT project_id, reporter_id FROM issues WHERE id = $1",
        issue_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("issue not found".into()))?;
    Ok((row.project_id, row.reporter_id))
}

/// Customer visibility rule for a single issue: the user reported it, or the
/// reporter shares at least one group with them.
pub async fn visible_to_user(pool: &PgPool, issue_id: Uuid, user_id: Uuid) -> Result<bool> {
    let visible = sqlx::query_scalar!(
        r#"SELECT EXISTS(
               SELECT 1 FROM issues i
               WHERE i.id = $1
                 AND (i.reporter_id = $2
                      OR EXISTS (SELECT 1 FROM group_members g1
                                 JOIN group_members g2 ON g2.group_id = g1.group_id
                                 WHERE g1.user_id = $2 AND g2.user_id = i.reporter_id))
           ) as "visible!""#,
        issue_id,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(visible)
}

/// Filtered, paginated issue list for a project. Backs both the table and the
/// Kanban board. Optional filters use the `($n IS NULL OR ...)` idiom so the
/// query stays compile-time checked. `visible_to` is Some(user) for customer
/// callers: rows are then limited to issues reported by that user or by a
/// group-mate (shared `group_members` row).
pub async fn list(
    pool: &PgPool,
    project_id: Uuid,
    project_key: &str,
    f: &IssueFilter,
    visible_to: Option<Uuid>,
) -> Result<IssueListResponse> {
    let page = f.page.unwrap_or(1).max(1);
    let page_size = f.page_size.unwrap_or(200).clamp(1, 500);
    let offset = (page - 1) * page_size;
    let q_like = f.q.as_ref().map(|s| format!("%{s}%"));
    // The board wants manual ordering; the list wants newest-first.
    let sort_board = f.sort.as_deref() == Some("board");

    let rows = sqlx::query!(
        r#"SELECT i.id, i.number, i.title, i.type, i.status, i.priority,
                  i.assignee_id, a.display_name as "assignee_name?",
                  i.created_at, i.updated_at,
                  COALESCE(
                      (SELECT max(al.created_at) FROM activity_log al
                       WHERE al.issue_id = i.id
                         AND al.action = 1            -- StatusChanged
                         AND al.new_value = i.status::text),
                      i.created_at
                  ) AS "status_since!"
           FROM issues i
           LEFT JOIN users a ON a.id = i.assignee_id
           WHERE i.project_id = $1
             AND ($2::smallint IS NULL OR i.status = $2)
             AND ($3::uuid     IS NULL OR i.assignee_id = $3)
             AND ($4::smallint IS NULL OR i.type = $4)
             AND ($5::smallint IS NULL OR i.priority = $5)
             AND ($6::text     IS NULL OR i.title ILIKE $6
                    OR ($10 || '-' || i.number::text) ILIKE $6)
             AND ($7::uuid     IS NULL OR EXISTS(
                    SELECT 1 FROM issue_labels il
                    WHERE il.issue_id = i.id AND il.label_id = $7))
             AND ($12::uuid    IS NULL OR i.reporter_id = $12
                    OR EXISTS (SELECT 1 FROM group_members g1
                               JOIN group_members g2 ON g2.group_id = g1.group_id
                               WHERE g1.user_id = $12 AND g2.user_id = i.reporter_id))
           ORDER BY (CASE WHEN $11 THEN i.board_position ELSE 0 END), i.number DESC
           LIMIT $8 OFFSET $9"#,
        project_id,
        f.status,
        f.assignee_id,
        f.r#type,
        f.priority,
        q_like,
        f.label_id,
        page_size,
        offset,
        project_key,
        sort_board,
        visible_to
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query!(
        r#"SELECT COUNT(*) as "count!"
           FROM issues i
           WHERE i.project_id = $1
             AND ($2::smallint IS NULL OR i.status = $2)
             AND ($3::uuid     IS NULL OR i.assignee_id = $3)
             AND ($4::smallint IS NULL OR i.type = $4)
             AND ($5::smallint IS NULL OR i.priority = $5)
             AND ($6::text     IS NULL OR i.title ILIKE $6
                    OR ($8 || '-' || i.number::text) ILIKE $6)
             AND ($7::uuid     IS NULL OR EXISTS(
                    SELECT 1 FROM issue_labels il
                    WHERE il.issue_id = i.id AND il.label_id = $7))
             AND ($9::uuid     IS NULL OR i.reporter_id = $9
                    OR EXISTS (SELECT 1 FROM group_members g1
                               JOIN group_members g2 ON g2.group_id = g1.group_id
                               WHERE g1.user_id = $9 AND g2.user_id = i.reporter_id))"#,
        project_id,
        f.status,
        f.assignee_id,
        f.r#type,
        f.priority,
        q_like,
        f.label_id,
        project_key,
        visible_to
    )
    .fetch_one(pool)
    .await?
    .count;

    // Batch-load labels for all returned issues.
    let issue_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let labels_by_issue = labels_for_issues(pool, &issue_ids).await?;

    let items = rows
        .into_iter()
        .map(|r| IssueListItem {
            key: format!("{}-{}", project_key, r.number),
            id: r.id,
            number: r.number,
            title: r.title,
            r#type: r.r#type,
            status: r.status,
            priority: r.priority,
            assignee_id: r.assignee_id,
            assignee_name: r.assignee_name,
            created_at: r.created_at,
            updated_at: r.updated_at,
            status_since: r.status_since,
            labels: labels_by_issue.get(&r.id).cloned().unwrap_or_default(),
        })
        .collect();

    Ok(IssueListResponse {
        items,
        total,
        page,
        page_size,
    })
}

/// Apply a partial update, recording an activity row per changed field, all in
/// one transaction. Returns the refreshed detail.
pub async fn update(
    pool: &PgPool,
    issue_id: Uuid,
    actor_id: Uuid,
    req: &UpdateIssueRequest,
) -> Result<IssueDetail> {
    let mut tx = pool.begin().await?;

    let cur = sqlx::query!(
        r#"SELECT title, description, type, status, priority, assignee_id
           FROM issues WHERE id = $1 FOR UPDATE"#,
        issue_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("issue not found".into()))?;

    let new_title = req.title.clone().unwrap_or_else(|| cur.title.clone());
    let new_description = match &req.description {
        Some(d) => Some(d.clone()),
        None => cur.description.clone(),
    };
    let new_type = req.r#type.unwrap_or(cur.r#type);
    let new_status = req.status.unwrap_or(cur.status);
    let new_priority = req.priority.unwrap_or(cur.priority);
    let new_assignee = match req.assignee_id {
        Some(v) => v,            // explicit set (possibly null)
        None => cur.assignee_id, // unchanged
    };

    // When the status changes outside the board's drag-reorder (e.g. from the
    // issue detail page), drop the card at the top of its new column so it has a
    // sensible board position instead of keeping the old column's rank.
    let new_position = if new_status != cur.status {
        sqlx::query_scalar!(
            "SELECT COALESCE(MIN(board_position), 1) - 1 FROM issues
             WHERE project_id = (SELECT project_id FROM issues WHERE id = $1) AND status = $2",
            issue_id,
            new_status
        )
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(0.0)
    } else {
        // Unchanged column: keep the current rank.
        sqlx::query_scalar!("SELECT board_position FROM issues WHERE id = $1", issue_id)
            .fetch_one(&mut *tx)
            .await?
    };

    sqlx::query!(
        r#"UPDATE issues SET
              title = $2, description = $3, type = $4, status = $5, priority = $6,
              assignee_id = $7, board_position = $8, updated_at = now()
           WHERE id = $1"#,
        issue_id,
        new_title,
        new_description,
        new_type,
        new_status,
        new_priority,
        new_assignee,
        new_position
    )
    .execute(&mut *tx)
    .await?;

    // Activity rows for the meaningful field changes.
    if new_status != cur.status {
        log_change(&mut tx, issue_id, actor_id, ActivityAction::StatusChanged, "status", Some(cur.status.to_string()), Some(new_status.to_string())).await?;
    }
    if new_assignee != cur.assignee_id {
        log_change(&mut tx, issue_id, actor_id, ActivityAction::AssigneeChanged, "assignee", cur.assignee_id.map(|u| u.to_string()), new_assignee.map(|u| u.to_string())).await?;
    }
    if new_priority != cur.priority {
        log_change(&mut tx, issue_id, actor_id, ActivityAction::PriorityChanged, "priority", Some(cur.priority.to_string()), Some(new_priority.to_string())).await?;
    }
    if new_type != cur.r#type {
        log_change(&mut tx, issue_id, actor_id, ActivityAction::TypeChanged, "type", Some(cur.r#type.to_string()), Some(new_type.to_string())).await?;
    }
    if new_title != cur.title {
        log_change(&mut tx, issue_id, actor_id, ActivityAction::TitleChanged, "title", Some(cur.title.clone()), Some(new_title.clone())).await?;
    }

    tx.commit().await?;
    get_detail(pool, issue_id).await
}

/// Reorder an issue within (or into) a status column for the Kanban board. The
/// new rank is the midpoint between its target neighbors; either may be absent
/// when dropped at an end. Logs a status change if the column changed.
pub async fn reorder(
    pool: &PgPool,
    issue_id: Uuid,
    actor_id: Uuid,
    req: &ReorderRequest,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    // Lock the row; we need its project (to scope neighbors) and old status.
    let cur = sqlx::query!(
        "SELECT project_id, status FROM issues WHERE id = $1 FOR UPDATE",
        issue_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("issue not found".into()))?;

    let before = neighbor_pos(&mut tx, cur.project_id, req.before_id).await?;
    let after = neighbor_pos(&mut tx, cur.project_id, req.after_id).await?;

    let new_pos = match (before, after) {
        (Some(b), Some(a)) if (b - a).abs() < 1e-6 => {
            // Fractional gap collapsed — renumber the column to integers and retry.
            renormalize(&mut tx, cur.project_id, req.status).await?;
            let b = neighbor_pos(&mut tx, cur.project_id, req.before_id)
                .await?
                .unwrap_or(0.0);
            let a = neighbor_pos(&mut tx, cur.project_id, req.after_id)
                .await?
                .unwrap_or(b + 2.0);
            (b + a) / 2.0
        }
        (Some(b), Some(a)) => (b + a) / 2.0,
        (Some(b), None) => b + 1.0, // dropped at the bottom
        (None, Some(a)) => a - 1.0, // dropped at the top
        (None, None) => 0.0,        // empty column
    };

    sqlx::query!(
        "UPDATE issues SET status = $2, board_position = $3, updated_at = now() WHERE id = $1",
        issue_id,
        req.status,
        new_pos
    )
    .execute(&mut *tx)
    .await?;

    if req.status != cur.status {
        log_change(
            &mut tx,
            issue_id,
            actor_id,
            ActivityAction::StatusChanged,
            "status",
            Some(cur.status.to_string()),
            Some(req.status.to_string()),
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, issue_id: Uuid) -> Result<()> {
    let res = sqlx::query!("DELETE FROM issues WHERE id = $1", issue_id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("issue not found".into()));
    }
    Ok(())
}

// ----------------------------- helpers -----------------------------

async fn log_change(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    issue_id: Uuid,
    actor_id: Uuid,
    action: ActivityAction,
    field: &str,
    old_value: Option<String>,
    new_value: Option<String>,
) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO activity_log (issue_id, actor_id, action, field, old_value, new_value)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        issue_id,
        actor_id,
        action.as_i16(),
        field,
        old_value,
        new_value
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Board position of a neighbor card (scoped to the project). `None` when no
/// neighbor id was given or it no longer exists.
async fn neighbor_pos(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: Uuid,
    id: Option<Uuid>,
) -> Result<Option<f64>> {
    let Some(id) = id else { return Ok(None) };
    let pos = sqlx::query_scalar!(
        "SELECT board_position FROM issues WHERE id = $1 AND project_id = $2",
        id,
        project_id
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(pos)
}

/// Renumber a column's positions to 1..n by their current order. Called when
/// fractional midpoints have collapsed below float precision.
async fn renormalize(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: Uuid,
    status: i16,
) -> Result<()> {
    sqlx::query!(
        r#"WITH ranked AS (
               SELECT id, row_number() OVER (ORDER BY board_position, number DESC) AS rn
               FROM issues WHERE project_id = $1 AND status = $2
           )
           UPDATE issues i SET board_position = r.rn
           FROM ranked r WHERE r.id = i.id"#,
        project_id,
        status
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn labels_for_issue(pool: &PgPool, issue_id: Uuid) -> Result<Vec<LabelRow>> {
    let rows = sqlx::query_as!(
        LabelRow,
        r#"SELECT l.id, l.project_id, l.name, l.color
           FROM issue_labels il
           JOIN labels l ON l.id = il.label_id
           WHERE il.issue_id = $1
           ORDER BY l.name"#,
        issue_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn labels_for_issues(
    pool: &PgPool,
    issue_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<LabelRow>>> {
    let mut map: HashMap<Uuid, Vec<LabelRow>> = HashMap::new();
    if issue_ids.is_empty() {
        return Ok(map);
    }
    let rows = sqlx::query!(
        r#"SELECT il.issue_id, l.id, l.project_id, l.name, l.color
           FROM issue_labels il
           JOIN labels l ON l.id = il.label_id
           WHERE il.issue_id = ANY($1)
           ORDER BY l.name"#,
        issue_ids
    )
    .fetch_all(pool)
    .await?;
    for r in rows {
        map.entry(r.issue_id).or_default().push(LabelRow {
            id: r.id,
            project_id: r.project_id,
            name: r.name,
            color: r.color,
        });
    }
    Ok(map)
}
