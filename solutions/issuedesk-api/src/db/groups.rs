use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{GroupRow, MemberRow},
};

pub async fn list(pool: &PgPool) -> Result<Vec<GroupRow>> {
    let rows = sqlx::query_as!(
        GroupRow,
        r#"SELECT g.id, g.name, g.description, g.created_by,
                  COUNT(gm.user_id) as "member_count!",
                  g.created_at, g.updated_at
           FROM groups g
           LEFT JOIN group_members gm ON gm.group_id = g.id
           GROUP BY g.id
           ORDER BY g.name"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
    created_by: Uuid,
) -> Result<GroupRow> {
    let row = sqlx::query_as!(
        GroupRow,
        r#"INSERT INTO groups (name, description, created_by)
           VALUES ($1, $2, $3)
           RETURNING id, name, description, created_by,
                     0::bigint as "member_count!", created_at, updated_at"#,
        name,
        description,
        created_by
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<GroupRow> {
    let row = sqlx::query_as!(
        GroupRow,
        r#"UPDATE groups SET
              name        = COALESCE($2, name),
              description = COALESCE($3, description),
              updated_at  = now()
           WHERE id = $1
           RETURNING id, name, description, created_by,
                     (SELECT COUNT(*) FROM group_members gm WHERE gm.group_id = groups.id) as "member_count!",
                     created_at, updated_at"#,
        id,
        name,
        description
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("group not found".into()))?;
    Ok(row)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let res = sqlx::query!("DELETE FROM groups WHERE id = $1", id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("group not found".into()));
    }
    Ok(())
}

// ----------------------------- members -----------------------------

/// Group roster. `role` carries the user's *global* role so the UI can badge
/// customer accounts.
pub async fn list_members(pool: &PgPool, group_id: Uuid) -> Result<Vec<MemberRow>> {
    let rows = sqlx::query_as!(
        MemberRow,
        r#"SELECT u.id as user_id, u.user_name, u.display_name, u.email,
                  u.role, gm.added_at
           FROM group_members gm
           JOIN users u ON u.id = gm.user_id
           WHERE gm.group_id = $1
           ORDER BY u.display_name"#,
        group_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn add_member(pool: &PgPool, group_id: Uuid, user_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO group_members (group_id, user_id)
           VALUES ($1, $2)
           ON CONFLICT (group_id, user_id) DO NOTHING"#,
        group_id,
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_member(pool: &PgPool, group_id: Uuid, user_id: Uuid) -> Result<()> {
    let res = sqlx::query!(
        "DELETE FROM group_members WHERE group_id = $1 AND user_id = $2",
        group_id,
        user_id
    )
    .execute(pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("membership not found".into()));
    }
    Ok(())
}
