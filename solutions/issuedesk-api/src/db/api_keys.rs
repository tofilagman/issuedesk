use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::Result, models::ApiKeyRow};

pub async fn list(pool: &PgPool) -> Result<Vec<ApiKeyRow>> {
    let rows = sqlx::query_as!(
        ApiKeyRow,
        r#"SELECT id, name, key_hash, prefix, created_by, last_used_at, created_at
           FROM api_keys ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(
    pool: &PgPool,
    name: &str,
    key_hash: &str,
    prefix: &str,
    created_by: Uuid,
) -> Result<ApiKeyRow> {
    let row = sqlx::query_as!(
        ApiKeyRow,
        r#"INSERT INTO api_keys (name, key_hash, prefix, created_by)
           VALUES ($1, $2, $3, $4)
           RETURNING id, name, key_hash, prefix, created_by, last_used_at, created_at"#,
        name,
        key_hash,
        prefix,
        created_by
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Look up an active key by the SHA-256 hash of its secret.
pub async fn find_by_hash(pool: &PgPool, key_hash: &str) -> Result<Option<ApiKeyRow>> {
    let row = sqlx::query_as!(
        ApiKeyRow,
        r#"SELECT id, name, key_hash, prefix, created_by, last_used_at, created_at
           FROM api_keys WHERE key_hash = $1"#,
        key_hash
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Stamp a key as used. Best-effort — failures are not fatal to the request.
pub async fn touch(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query!("UPDATE api_keys SET last_used_at = now() WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let res = sqlx::query!("DELETE FROM api_keys WHERE id = $1", id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(crate::error::AppError::NotFound("api key not found".into()));
    }
    Ok(())
}
