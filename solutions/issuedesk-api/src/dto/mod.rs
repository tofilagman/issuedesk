//! Request/response shapes. All JSON is camelCase. Requests validate via the
//! `validator` crate; responses are plain serializable structs.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::models::UserDto;

// ----------------------------- auth -----------------------------

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SignInRequest {
    #[validate(length(min = 1, message = "required"))]
    pub user_name: String,
    #[validate(length(min = 1, message = "required"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub token: String,
    pub user: UserDto,
}

// ----------------------------- users -----------------------------

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 64, message = "1-64 chars"))]
    pub user_name: String,
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    #[validate(length(min = 1, max = 128, message = "1-128 chars"))]
    pub display_name: String,
    #[validate(length(min = 6, message = "min 6 chars"))]
    pub password: String,
    /// 0=member, 1=admin
    pub role: Option<i16>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 128, message = "1-128 chars"))]
    pub display_name: Option<String>,
    #[validate(email(message = "must be a valid email"))]
    pub email: Option<String>,
    pub role: Option<i16>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    #[validate(length(min = 6, message = "min 6 chars"))]
    pub password: String,
}

// ----------------------------- projects -----------------------------

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    /// Validated against PROJECT_KEY_RE in the handler.
    #[validate(length(min = 2, max = 10, message = "2-10 chars"))]
    pub key: String,
    #[validate(length(min = 1, max = 128, message = "1-128 chars"))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    #[validate(length(min = 1, max = 128, message = "1-128 chars"))]
    pub name: Option<String>,
    pub description: Option<String>,
}

// ----------------------------- members -----------------------------

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    /// 0=member, 1=lead
    pub role: Option<i16>,
}

// ----------------------------- issues -----------------------------

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueRequest {
    #[validate(length(min = 1, max = 256, message = "1-256 chars"))]
    pub title: String,
    pub description: Option<String>,
    /// 0 bug,1 task,2 story,3 epic
    pub r#type: Option<i16>,
    /// 0 low,1 medium,2 high,3 urgent
    pub priority: Option<i16>,
    pub assignee_id: Option<Uuid>,
    pub label_ids: Option<Vec<Uuid>>,
}

/// Partial update — every field optional. Status changes here drive the Kanban
/// board drop handler.
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIssueRequest {
    #[validate(length(min = 1, max = 256, message = "1-256 chars"))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<i16>,
    pub status: Option<i16>,
    pub priority: Option<i16>,
    /// Wrap in an extra Option so clients can explicitly set assignee to null
    /// (unassign) vs. omitting the field (leave unchanged).
    #[serde(default, deserialize_with = "double_option")]
    pub assignee_id: Option<Option<Uuid>>,
}

/// Query parameters for the issue list (backs both the table and the board).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueFilter {
    pub status: Option<i16>,
    pub assignee_id: Option<Uuid>,
    pub r#type: Option<i16>,
    pub priority: Option<i16>,
    pub label_id: Option<Uuid>,
    pub q: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueListItem {
    pub id: Uuid,
    pub key: String, // e.g. "WAT-1"
    pub number: i64,
    pub title: String,
    pub r#type: i16,
    pub status: i16,
    pub priority: i16,
    pub assignee_id: Option<Uuid>,
    pub assignee_name: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub labels: Vec<crate::models::LabelRow>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueListResponse {
    pub items: Vec<IssueListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueDetail {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_key: String,
    pub key: String,
    pub number: i64,
    pub title: String,
    pub description: Option<String>,
    pub r#type: i16,
    pub status: i16,
    pub priority: i16,
    pub assignee_id: Option<Uuid>,
    pub assignee_name: Option<String>,
    pub reporter_id: Uuid,
    pub reporter_name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub labels: Vec<crate::models::LabelRow>,
}

// ----------------------------- comments -----------------------------

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentRequest {
    #[validate(length(min = 1, message = "required"))]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommentRequest {
    #[validate(length(min = 1, message = "required"))]
    pub body: String,
}

// ----------------------------- labels -----------------------------

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateLabelRequest {
    #[validate(length(min = 1, max = 48, message = "1-48 chars"))]
    pub name: String,
    /// Validated against COLOR_RE in the handler.
    pub color: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachLabelRequest {
    pub label_id: Uuid,
}

// ----------------------------- helpers -----------------------------

use std::sync::LazyLock;
use regex::Regex;

pub static PROJECT_KEY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][A-Z0-9]{1,9}$").unwrap());
pub static COLOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap());

/// Deserialize a field into `Option<Option<T>>` so we can distinguish
/// "field absent" (None) from "field present and null" (Some(None)).
fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::deserialize(de)?))
}
