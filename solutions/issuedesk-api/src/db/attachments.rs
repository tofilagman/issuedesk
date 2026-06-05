use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{enums::ActivityAction, AttachmentRow},
};

pub async fn list(pool: &PgPool, issue_id: Uuid) -> Result<Vec<AttachmentRow>> {
    let rows = sqlx::query_as!(
        AttachmentRow,
        r#"SELECT id, issue_id, filename, stored_path, size_bytes, mime_type,
                  uploaded_by, created_at
           FROM attachments WHERE issue_id = $1 ORDER BY created_at DESC"#,
        issue_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(
    pool: &PgPool,
    issue_id: Uuid,
    filename: &str,
    stored_path: &str,
    size_bytes: i64,
    mime_type: &str,
    uploaded_by: Uuid,
) -> Result<AttachmentRow> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as!(
        AttachmentRow,
        r#"INSERT INTO attachments (issue_id, filename, stored_path, size_bytes, mime_type, uploaded_by)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, issue_id, filename, stored_path, size_bytes, mime_type, uploaded_by, created_at"#,
        issue_id,
        filename,
        stored_path,
        size_bytes,
        mime_type,
        uploaded_by
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO activity_log (issue_id, actor_id, action, new_value) VALUES ($1, $2, $3, $4)",
        issue_id,
        uploaded_by,
        ActivityAction::AttachmentAdded.as_i16(),
        filename
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<AttachmentRow> {
    let row = sqlx::query_as!(
        AttachmentRow,
        r#"SELECT id, issue_id, filename, stored_path, size_bytes, mime_type,
                  uploaded_by, created_at
           FROM attachments WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("attachment not found".into()))?;
    Ok(row)
}

pub async fn delete(pool: &PgPool, id: Uuid, actor_id: Uuid) -> Result<AttachmentRow> {
    // Return the row so the caller can remove the file from disk.
    let row = get(pool, id).await?;
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM attachments WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "INSERT INTO activity_log (issue_id, actor_id, action, old_value) VALUES ($1, $2, $3, $4)",
        row.issue_id,
        actor_id,
        ActivityAction::AttachmentRemoved.as_i16(),
        row.filename
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}
