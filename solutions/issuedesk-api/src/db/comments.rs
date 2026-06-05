use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{enums::ActivityAction, CommentRow},
};

pub async fn list(pool: &PgPool, issue_id: Uuid) -> Result<Vec<CommentRow>> {
    let rows = sqlx::query_as!(
        CommentRow,
        r#"SELECT c.id, c.issue_id, c.author_id, u.display_name as author_name,
                  c.body, c.created_at, c.updated_at
           FROM comments c
           JOIN users u ON u.id = c.author_id
           WHERE c.issue_id = $1
           ORDER BY c.created_at"#,
        issue_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(pool: &PgPool, issue_id: Uuid, author_id: Uuid, body: &str) -> Result<CommentRow> {
    let mut tx = pool.begin().await?;
    let id = sqlx::query!(
        "INSERT INTO comments (issue_id, author_id, body) VALUES ($1, $2, $3) RETURNING id",
        issue_id,
        author_id,
        body
    )
    .fetch_one(&mut *tx)
    .await?
    .id;

    sqlx::query!(
        "INSERT INTO activity_log (issue_id, actor_id, action) VALUES ($1, $2, $3)",
        issue_id,
        author_id,
        ActivityAction::Commented.as_i16()
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    // Re-read with author name.
    let row = sqlx::query_as!(
        CommentRow,
        r#"SELECT c.id, c.issue_id, c.author_id, u.display_name as author_name,
                  c.body, c.created_at, c.updated_at
           FROM comments c JOIN users u ON u.id = c.author_id
           WHERE c.id = $1"#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Returns (author_id) of a comment for authorization checks.
pub async fn author_of(pool: &PgPool, comment_id: Uuid) -> Result<Uuid> {
    let row = sqlx::query!("SELECT author_id FROM comments WHERE id = $1", comment_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("comment not found".into()))?;
    Ok(row.author_id)
}

pub async fn update(pool: &PgPool, comment_id: Uuid, body: &str) -> Result<CommentRow> {
    sqlx::query!(
        "UPDATE comments SET body = $2, updated_at = now() WHERE id = $1",
        comment_id,
        body
    )
    .execute(pool)
    .await?;
    let row = sqlx::query_as!(
        CommentRow,
        r#"SELECT c.id, c.issue_id, c.author_id, u.display_name as author_name,
                  c.body, c.created_at, c.updated_at
           FROM comments c JOIN users u ON u.id = c.author_id
           WHERE c.id = $1"#,
        comment_id
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete(pool: &PgPool, comment_id: Uuid) -> Result<()> {
    let res = sqlx::query!("DELETE FROM comments WHERE id = $1", comment_id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("comment not found".into()));
    }
    Ok(())
}
