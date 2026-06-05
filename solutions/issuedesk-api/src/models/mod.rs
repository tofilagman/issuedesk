pub mod enums;

use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

/// Database row for a user. `password_hash` is never serialized to clients.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub user_name: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub role: i16,
    pub is_active: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Public-facing user shape (no password hash).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: Uuid,
    pub user_name: String,
    pub email: String,
    pub display_name: String,
    pub role: i16,
    pub is_active: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<UserRow> for UserDto {
    fn from(u: UserRow) -> Self {
        Self {
            id: u.id,
            user_name: u.user_name,
            email: u.email,
            display_name: u.display_name,
            role: u.role,
            is_active: u.is_active,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRow {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub issue_seq: i64,
    pub created_by: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentRow {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub body: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentRow {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub filename: String,
    pub stored_path: String,
    pub size_bytes: i64,
    pub mime_type: String,
    pub uploaded_by: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRow {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub actor_id: Uuid,
    pub actor_name: String,
    pub action: i16,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberRow {
    pub user_id: Uuid,
    pub user_name: String,
    pub display_name: String,
    pub email: String,
    pub role: i16,
    #[serde(with = "time::serde::rfc3339")]
    pub added_at: OffsetDateTime,
}
