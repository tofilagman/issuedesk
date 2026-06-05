use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::Result, models::UserRow};

pub async fn find_by_username(pool: &PgPool, user_name: &str) -> Result<Option<UserRow>> {
    let row = sqlx::query_as!(
        UserRow,
        r#"SELECT id, user_name, email, display_name, password_hash, role, is_active,
                  created_at, updated_at
           FROM users WHERE user_name = $1"#,
        user_name
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UserRow>> {
    let row = sqlx::query_as!(
        UserRow,
        r#"SELECT id, user_name, email, display_name, password_hash, role, is_active,
                  created_at, updated_at
           FROM users WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list(pool: &PgPool) -> Result<Vec<UserRow>> {
    let rows = sqlx::query_as!(
        UserRow,
        r#"SELECT id, user_name, email, display_name, password_hash, role, is_active,
                  created_at, updated_at
           FROM users ORDER BY user_name"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(
    pool: &PgPool,
    user_name: &str,
    email: &str,
    display_name: &str,
    password_hash: &str,
    role: i16,
) -> Result<UserRow> {
    let row = sqlx::query_as!(
        UserRow,
        r#"INSERT INTO users (user_name, email, display_name, password_hash, role)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, user_name, email, display_name, password_hash, role, is_active,
                     created_at, updated_at"#,
        user_name,
        email,
        display_name,
        password_hash,
        role
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    display_name: Option<&str>,
    email: Option<&str>,
    role: Option<i16>,
    is_active: Option<bool>,
) -> Result<UserRow> {
    // COALESCE keeps the existing value when the argument is NULL (field omitted).
    let row = sqlx::query_as!(
        UserRow,
        r#"UPDATE users SET
              display_name = COALESCE($2, display_name),
              email        = COALESCE($3, email),
              role         = COALESCE($4, role),
              is_active    = COALESCE($5, is_active),
              updated_at   = now()
           WHERE id = $1
           RETURNING id, user_name, email, display_name, password_hash, role, is_active,
                     created_at, updated_at"#,
        id,
        display_name,
        email,
        role,
        is_active
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn set_password(pool: &PgPool, id: Uuid, password_hash: &str) -> Result<()> {
    let res = sqlx::query!(
        "UPDATE users SET password_hash = $2, updated_at = now() WHERE id = $1",
        id,
        password_hash
    )
    .execute(pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(crate::error::AppError::NotFound("user not found".into()));
    }
    Ok(())
}

pub async fn count(pool: &PgPool) -> Result<i64> {
    let row = sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM users"#)
        .fetch_one(pool)
        .await?;
    Ok(row.count)
}
