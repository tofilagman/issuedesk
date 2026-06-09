use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dto::{AssigneeStat, ProjectStats, ProjectSummary, SystemStats},
    error::Result,
};

/// Fold (index, count) rows into a fixed 4-element vector indexed by enum value.
fn fold4(pairs: &[(i16, i64)]) -> Vec<i64> {
    let mut v = vec![0i64; 4];
    for &(idx, count) in pairs {
        if (0..4).contains(&idx) {
            v[idx as usize] = count;
        }
    }
    v
}

pub async fn project_stats(pool: &PgPool, project_id: Uuid) -> Result<ProjectStats> {
    let status_rows = sqlx::query!(
        r#"SELECT status as "k!", COUNT(*) as "c!" FROM issues WHERE project_id = $1 GROUP BY status"#,
        project_id
    )
    .fetch_all(pool)
    .await?;
    let by_status = fold4(&status_rows.iter().map(|r| (r.k, r.c)).collect::<Vec<_>>());

    let type_rows = sqlx::query!(
        r#"SELECT type as "k!", COUNT(*) as "c!" FROM issues WHERE project_id = $1 GROUP BY type"#,
        project_id
    )
    .fetch_all(pool)
    .await?;
    let by_type = fold4(&type_rows.iter().map(|r| (r.k, r.c)).collect::<Vec<_>>());

    let priority_rows = sqlx::query!(
        r#"SELECT priority as "k!", COUNT(*) as "c!" FROM issues WHERE project_id = $1 GROUP BY priority"#,
        project_id
    )
    .fetch_all(pool)
    .await?;
    let by_priority = fold4(&priority_rows.iter().map(|r| (r.k, r.c)).collect::<Vec<_>>());

    let total: i64 = by_status.iter().sum();
    let done = by_status[3];
    let open = total - done;

    let unassigned = sqlx::query!(
        r#"SELECT COUNT(*) as "c!" FROM issues WHERE project_id = $1 AND assignee_id IS NULL"#,
        project_id
    )
    .fetch_one(pool)
    .await?
    .c;

    let created_last7 = sqlx::query!(
        r#"SELECT COUNT(*) as "c!" FROM issues
           WHERE project_id = $1 AND created_at > now() - interval '7 days'"#,
        project_id
    )
    .fetch_one(pool)
    .await?
    .c;

    // Issues moved to "Done" (status 3) in the last 7 days, per the activity log.
    let resolved_last7 = sqlx::query!(
        r#"SELECT COUNT(*) as "c!" FROM activity_log al
           JOIN issues i ON i.id = al.issue_id
           WHERE i.project_id = $1 AND al.action = 1 AND al.new_value = '3'
             AND al.created_at > now() - interval '7 days'"#,
        project_id
    )
    .fetch_one(pool)
    .await?
    .c;

    let assignee_rows = sqlx::query!(
        r#"SELECT i.assignee_id as "user_id!", u.display_name, COUNT(*) as "c!"
           FROM issues i JOIN users u ON u.id = i.assignee_id
           WHERE i.project_id = $1
           GROUP BY i.assignee_id, u.display_name
           ORDER BY COUNT(*) DESC, u.display_name
           LIMIT 8"#,
        project_id
    )
    .fetch_all(pool)
    .await?;
    let by_assignee = assignee_rows
        .into_iter()
        .map(|r| AssigneeStat {
            user_id: r.user_id,
            display_name: r.display_name,
            count: r.c,
        })
        .collect();

    Ok(ProjectStats {
        total,
        open,
        done,
        unassigned,
        created_last7,
        resolved_last7,
        by_status,
        by_type,
        by_priority,
        by_assignee,
    })
}

pub async fn system_stats(pool: &PgPool, project_ids: &[Uuid]) -> Result<SystemStats> {
    let status_rows = sqlx::query!(
        r#"SELECT status as "k!", COUNT(*) as "c!" FROM issues
           WHERE project_id = ANY($1) GROUP BY status"#,
        project_ids
    )
    .fetch_all(pool)
    .await?;
    let by_status = fold4(&status_rows.iter().map(|r| (r.k, r.c)).collect::<Vec<_>>());

    let type_rows = sqlx::query!(
        r#"SELECT type as "k!", COUNT(*) as "c!" FROM issues
           WHERE project_id = ANY($1) GROUP BY type"#,
        project_ids
    )
    .fetch_all(pool)
    .await?;
    let by_type = fold4(&type_rows.iter().map(|r| (r.k, r.c)).collect::<Vec<_>>());

    let priority_rows = sqlx::query!(
        r#"SELECT priority as "k!", COUNT(*) as "c!" FROM issues
           WHERE project_id = ANY($1) GROUP BY priority"#,
        project_ids
    )
    .fetch_all(pool)
    .await?;
    let by_priority = fold4(&priority_rows.iter().map(|r| (r.k, r.c)).collect::<Vec<_>>());

    let issues: i64 = by_status.iter().sum();
    let done = by_status[3];
    let open = issues - done;
    let projects = project_ids.len() as i64;

    let users = sqlx::query!(r#"SELECT COUNT(*) as "c!" FROM users WHERE is_active = true"#)
        .fetch_one(pool)
        .await?
        .c;

    let created_last7 = sqlx::query!(
        r#"SELECT COUNT(*) as "c!" FROM issues
           WHERE project_id = ANY($1) AND created_at > now() - interval '7 days'"#,
        project_ids
    )
    .fetch_one(pool)
    .await?
    .c;

    let top_rows = sqlx::query!(
        r#"SELECT p.key, p.name,
                  COUNT(i.id) as "total!",
                  COUNT(i.id) FILTER (WHERE i.status = 3) as "done!"
           FROM projects p
           LEFT JOIN issues i ON i.project_id = p.id
           WHERE p.id = ANY($1)
           GROUP BY p.id, p.key, p.name
           ORDER BY COUNT(i.id) DESC, p.key
           LIMIT 8"#,
        project_ids
    )
    .fetch_all(pool)
    .await?;
    let top_projects = top_rows
        .into_iter()
        .map(|r| ProjectSummary {
            key: r.key,
            name: r.name,
            total: r.total,
            done: r.done,
        })
        .collect();

    Ok(SystemStats {
        projects,
        issues,
        users,
        open,
        done,
        created_last7,
        by_status,
        by_type,
        by_priority,
        top_projects,
    })
}
