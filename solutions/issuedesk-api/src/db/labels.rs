use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{enums::ActivityAction, LabelRow},
};

pub async fn list(pool: &PgPool, project_id: Uuid) -> Result<Vec<LabelRow>> {
    let rows = sqlx::query_as!(
        LabelRow,
        "SELECT id, project_id, name, color FROM labels WHERE project_id = $1 ORDER BY name",
        project_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(pool: &PgPool, project_id: Uuid, name: &str, color: &str) -> Result<LabelRow> {
    let row = sqlx::query_as!(
        LabelRow,
        "INSERT INTO labels (project_id, name, color) VALUES ($1, $2, $3)
         RETURNING id, project_id, name, color",
        project_id,
        name,
        color
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn project_of(pool: &PgPool, label_id: Uuid) -> Result<Uuid> {
    let row = sqlx::query!("SELECT project_id FROM labels WHERE id = $1", label_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("label not found".into()))?;
    Ok(row.project_id)
}

pub async fn delete(pool: &PgPool, label_id: Uuid) -> Result<()> {
    let res = sqlx::query!("DELETE FROM labels WHERE id = $1", label_id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("label not found".into()));
    }
    Ok(())
}

pub async fn attach(pool: &PgPool, issue_id: Uuid, label_id: Uuid, actor_id: Uuid) -> Result<()> {
    let mut tx = pool.begin().await?;
    let inserted = sqlx::query!(
        "INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
        issue_id,
        label_id
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if inserted > 0 {
        sqlx::query!(
            "INSERT INTO activity_log (issue_id, actor_id, action, new_value) VALUES ($1, $2, $3, $4)",
            issue_id,
            actor_id,
            ActivityAction::LabelAdded.as_i16(),
            label_id.to_string()
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn detach(pool: &PgPool, issue_id: Uuid, label_id: Uuid, actor_id: Uuid) -> Result<()> {
    let mut tx = pool.begin().await?;
    let removed = sqlx::query!(
        "DELETE FROM issue_labels WHERE issue_id = $1 AND label_id = $2",
        issue_id,
        label_id
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if removed > 0 {
        sqlx::query!(
            "INSERT INTO activity_log (issue_id, actor_id, action, old_value) VALUES ($1, $2, $3, $4)",
            issue_id,
            actor_id,
            ActivityAction::LabelRemoved.as_i16(),
            label_id.to_string()
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
