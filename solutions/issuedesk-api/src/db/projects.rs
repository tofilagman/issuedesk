use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{MemberRow, ProjectRow},
};

pub async fn create(
    pool: &PgPool,
    key: &str,
    name: &str,
    description: Option<&str>,
    created_by: Uuid,
) -> Result<ProjectRow> {
    let mut tx = pool.begin().await?;
    let project = sqlx::query_as!(
        ProjectRow,
        r#"INSERT INTO projects (key, name, description, created_by)
           VALUES ($1, $2, $3, $4)
           RETURNING id, key, name, description, issue_seq, created_by, created_at, updated_at"#,
        key,
        name,
        description,
        created_by
    )
    .fetch_one(&mut *tx)
    .await?;

    // Creator is automatically a project lead.
    sqlx::query!(
        "INSERT INTO project_members (project_id, user_id, role) VALUES ($1, $2, 1)",
        project.id,
        created_by
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(project)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<ProjectRow> {
    let row = sqlx::query_as!(
        ProjectRow,
        r#"SELECT id, key, name, description, issue_seq, created_by, created_at, updated_at
           FROM projects WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("project not found".into()))?;
    Ok(row)
}

/// Projects visible to a user: all of them for admins, otherwise the ones they
/// are a member of.
pub async fn list_visible(pool: &PgPool, user_id: Uuid, is_admin: bool) -> Result<Vec<ProjectRow>> {
    let rows = if is_admin {
        sqlx::query_as!(
            ProjectRow,
            r#"SELECT id, key, name, description, issue_seq, created_by, created_at, updated_at
               FROM projects ORDER BY key"#
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as!(
            ProjectRow,
            r#"SELECT p.id, p.key, p.name, p.description, p.issue_seq, p.created_by,
                      p.created_at, p.updated_at
               FROM projects p
               JOIN project_members m ON m.project_id = p.id
               WHERE m.user_id = $1
               ORDER BY p.key"#,
            user_id
        )
        .fetch_all(pool)
        .await?
    };
    Ok(rows)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
) -> Result<ProjectRow> {
    let row = sqlx::query_as!(
        ProjectRow,
        r#"UPDATE projects SET
              name        = COALESCE($2, name),
              description = COALESCE($3, description),
              updated_at  = now()
           WHERE id = $1
           RETURNING id, key, name, description, issue_seq, created_by, created_at, updated_at"#,
        id,
        name,
        description
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let res = sqlx::query!("DELETE FROM projects WHERE id = $1", id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("project not found".into()));
    }
    Ok(())
}

/// True if the user is a member of the project.
pub async fn is_member(pool: &PgPool, project_id: Uuid, user_id: Uuid) -> Result<bool> {
    let row = sqlx::query!(
        r#"SELECT EXISTS(
              SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2
           ) as "exists!""#,
        project_id,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(row.exists)
}

// ----------------------------- members -----------------------------

pub async fn list_members(pool: &PgPool, project_id: Uuid) -> Result<Vec<MemberRow>> {
    let rows = sqlx::query_as!(
        MemberRow,
        r#"SELECT u.id as user_id, u.user_name, u.display_name, u.email,
                  m.role, m.added_at
           FROM project_members m
           JOIN users u ON u.id = m.user_id
           WHERE m.project_id = $1
           ORDER BY u.display_name"#,
        project_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn add_member(pool: &PgPool, project_id: Uuid, user_id: Uuid, role: i16) -> Result<()> {
    sqlx::query!(
        r#"INSERT INTO project_members (project_id, user_id, role)
           VALUES ($1, $2, $3)
           ON CONFLICT (project_id, user_id) DO UPDATE SET role = EXCLUDED.role"#,
        project_id,
        user_id,
        role
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_member(pool: &PgPool, project_id: Uuid, user_id: Uuid) -> Result<()> {
    let res = sqlx::query!(
        "DELETE FROM project_members WHERE project_id = $1 AND user_id = $2",
        project_id,
        user_id
    )
    .execute(pool)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("membership not found".into()));
    }
    Ok(())
}
