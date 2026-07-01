use axum::{
    extract::{Multipart, Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use std::fmt::Write as _;
use time::format_description::well_known::Rfc3339;

use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::AuthUser,
    db,
    dto::{CreateCommentRequest, TicketBundle, TicketQuery, UpdateCommentRequest},
    error::{AppError, Result},
    handlers::attachments,
    models::{AttachmentRow, CommentRow},
    state::AppState,
};

const STATUS: [&str; 5] = ["To Do", "In Progress", "Implemented", "In Review", "Done"];
const TYPE: [&str; 4] = ["Bug", "Task", "Story", "Epic"];
const PRIORITY: [&str; 4] = ["Low", "Medium", "High", "Urgent"];
const LINK_TYPE: [&str; 5] = [
    "Relates to",
    "Blocks",
    "Is blocked by",
    "Duplicates",
    "Is duplicated by",
];
const ACTION: [&str; 13] = [
    "created",
    "changed status",
    "changed assignee",
    "commented",
    "added label",
    "removed label",
    "added attachment",
    "removed attachment",
    "changed priority",
    "changed type",
    "changed title",
    "added link",
    "removed link",
];

fn label(table: &[&str], i: i16) -> String {
    table
        .get(i as usize)
        .map(|s| s.to_string())
        .unwrap_or_else(|| i.to_string())
}

/// `GET /api/tickets/{KEY-number}` — fetch a whole ticket by its public key
/// (e.g. `WAT-1`). `?format=md` returns Markdown; otherwise JSON.
///
/// This is the URL-friendly entry point for relaying a ticket to an external
/// client: an API key plus the key from the ticket URL is enough.
pub async fn get(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Query(q): Query<TicketQuery>,
) -> Result<Response> {
    let bundle = load(&state, &user, &slug).await?;
    if q.format.as_deref() == Some("md") {
        let md = to_markdown(&bundle);
        Ok(([(header::CONTENT_TYPE, "text/markdown; charset=utf-8")], md).into_response())
    } else {
        Ok(Json(bundle).into_response())
    }
}

/// `POST /api/tickets/{KEY-number}/comments` — add a comment to a ticket by its
/// public key (e.g. `WAT-1`). The slug-addressed counterpart to
/// `POST /api/issues/{id}/comments`, so a relay client (API key) can comment
/// back with only the key from the ticket URL.
pub async fn comment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<CommentRow>> {
    req.validate()?;
    let issue_id = resolve(&state, &user, &slug).await?;
    let row = db::comments::create(&state.pool, issue_id, user.id(), &req.body).await?;
    Ok(Json(row))
}

/// `PATCH /api/tickets/{KEY-number}/comments/{commentId}` — edit a comment on a
/// ticket addressed by its public key. Slug-addressed counterpart to
/// `PATCH /api/comments/{id}`. The caller must own the comment (or be an admin).
pub async fn comment_update(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, comment_id)): Path<(String, Uuid)>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<Json<CommentRow>> {
    req.validate()?;
    let issue_id = resolve(&state, &user, &slug).await?;
    authorize_comment(&state, &user, issue_id, comment_id).await?;
    let row = db::comments::update(&state.pool, comment_id, &req.body).await?;
    Ok(Json(row))
}

/// `DELETE /api/tickets/{KEY-number}/comments/{commentId}` — delete a comment on
/// a ticket addressed by its public key. Slug-addressed counterpart to
/// `DELETE /api/comments/{id}`. The caller must own the comment (or be an admin).
pub async fn comment_delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path((slug, comment_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>> {
    let issue_id = resolve(&state, &user, &slug).await?;
    authorize_comment(&state, &user, issue_id, comment_id).await?;
    db::comments::delete(&state.pool, comment_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// `GET /api/tickets/{KEY-number}/attachments` — list a ticket's attachments by
/// its public key. Slug-addressed counterpart to
/// `GET /api/issues/{id}/attachments`.
pub async fn attachment_list(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<Vec<AttachmentRow>>> {
    let issue_id = resolve(&state, &user, &slug).await?;
    let rows = db::attachments::list(&state.pool, issue_id).await?;
    Ok(Json(rows))
}

/// `POST /api/tickets/{KEY-number}/attachments` — upload a file to a ticket by
/// its public key (multipart/form-data, first file field). Slug-addressed
/// counterpart to `POST /api/issues/{id}/attachments`, so a relay client (API
/// key) can attach a file when commenting back with only the key from the URL.
/// The returned attachment id downloads via `GET /api/attachments/{id}`.
pub async fn attachment_upload(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    multipart: Multipart,
) -> Result<Json<AttachmentRow>> {
    let issue_id = resolve(&state, &user, &slug).await?;
    let row = attachments::store_upload(&state, issue_id, user.id(), multipart).await?;
    Ok(Json(row))
}

/// Confirm a comment exists, belongs to the resolved ticket, and that the caller
/// is allowed to mutate it (its author, or an admin).
async fn authorize_comment(
    state: &AppState,
    user: &AuthUser,
    issue_id: Uuid,
    comment_id: Uuid,
) -> Result<()> {
    let (comment_issue, author) = db::comments::issue_and_author(&state.pool, comment_id).await?;
    if comment_issue != issue_id {
        return Err(AppError::NotFound("comment not found on this ticket".into()));
    }
    if !user.is_admin() && user.id() != author {
        return Err(AppError::Forbidden("not your comment".into()));
    }
    Ok(())
}

/// Parse a `KEY-number` slug, authorize the caller against its project, and
/// return the issue id. Shared by the ticket read and comment endpoints.
async fn resolve(state: &AppState, user: &AuthUser, slug: &str) -> Result<Uuid> {
    // Split "WAT-1" into key + number on the final hyphen.
    let (key, num) = slug
        .rsplit_once('-')
        .ok_or_else(|| AppError::BadRequest("ticket key must look like KEY-number".into()))?;
    let number: i64 = num
        .parse()
        .map_err(|_| AppError::BadRequest("ticket number must be an integer".into()))?;

    let project = db::projects::find_by_key(&state.pool, &key.to_uppercase()).await?;
    db::authorize_project(&state.pool, user, project.id).await?;

    db::issues::id_by_number(&state.pool, project.id, number).await
}

async fn load(state: &AppState, user: &AuthUser, slug: &str) -> Result<TicketBundle> {
    let issue_id = resolve(state, user, slug).await?;
    let issue = db::issues::get_detail(&state.pool, issue_id).await?;
    let comments = db::comments::list(&state.pool, issue_id).await?;
    let activity = db::activity::list(&state.pool, issue_id).await?;
    let links = db::links::list(&state.pool, issue_id).await?;
    let attachments = db::attachments::list(&state.pool, issue_id).await?;

    Ok(TicketBundle {
        issue,
        comments,
        activity,
        links,
        attachments,
    })
}

fn fmt_time(t: time::OffsetDateTime) -> String {
    t.format(&Rfc3339).unwrap_or_default()
}

/// Human-readable byte size for the Markdown attachment list (e.g. "1.4 MB").
fn human_size(bytes: i64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

fn to_markdown(b: &TicketBundle) -> String {
    let i = &b.issue;
    let mut s = String::new();

    let _ = writeln!(s, "# {} — {}", i.key, i.title);
    let _ = writeln!(s);
    let _ = writeln!(s, "- **Type:** {}", label(&TYPE, i.r#type));
    let _ = writeln!(s, "- **Status:** {}", label(&STATUS, i.status));
    let _ = writeln!(s, "- **Priority:** {}", label(&PRIORITY, i.priority));
    let _ = writeln!(
        s,
        "- **Assignee:** {}",
        i.assignee_name.as_deref().unwrap_or("Unassigned")
    );
    let _ = writeln!(s, "- **Reporter:** {}", i.reporter_name);
    if !i.labels.is_empty() {
        let names: Vec<&str> = i.labels.iter().map(|l| l.name.as_str()).collect();
        let _ = writeln!(s, "- **Labels:** {}", names.join(", "));
    }
    let _ = writeln!(s, "- **Created:** {}", fmt_time(i.created_at));
    let _ = writeln!(s, "- **Updated:** {}", fmt_time(i.updated_at));
    let _ = writeln!(s);

    let _ = writeln!(s, "## Description");
    let _ = writeln!(s);
    match i.description.as_deref() {
        Some(d) if !d.trim().is_empty() => {
            // Stored as rich HTML; passed through verbatim (Claude reads HTML).
            let _ = writeln!(s, "{d}");
        }
        _ => {
            let _ = writeln!(s, "_No description._");
        }
    }
    let _ = writeln!(s);

    if !b.links.is_empty() {
        let _ = writeln!(s, "## Linked issues");
        let _ = writeln!(s);
        for l in &b.links {
            let _ = writeln!(
                s,
                "- {} **{}** {} _({})_",
                label(&LINK_TYPE, l.link_type),
                l.key,
                l.title,
                label(&STATUS, l.status)
            );
        }
        let _ = writeln!(s);
    }

    if !b.attachments.is_empty() {
        let _ = writeln!(s, "## Attachments ({})", b.attachments.len());
        let _ = writeln!(s);
        for a in &b.attachments {
            let _ = writeln!(
                s,
                "- [{}](/api/attachments/{}) — {} ({})",
                a.filename,
                a.id,
                human_size(a.size_bytes),
                a.mime_type
            );
        }
        let _ = writeln!(s);
    }

    let _ = writeln!(s, "## Comments ({})", b.comments.len());
    let _ = writeln!(s);
    if b.comments.is_empty() {
        let _ = writeln!(s, "_No comments._");
        let _ = writeln!(s);
    } else {
        for c in &b.comments {
            let _ = writeln!(s, "### {} — {}", c.author_name, fmt_time(c.created_at));
            let _ = writeln!(s);
            let _ = writeln!(s, "{}", c.body);
            let _ = writeln!(s);
        }
    }

    if !b.activity.is_empty() {
        let _ = writeln!(s, "## Activity");
        let _ = writeln!(s);
        for a in &b.activity {
            let detail = match (&a.old_value, &a.new_value) {
                (Some(o), Some(n)) => format!(" ({o} → {n})"),
                (None, Some(n)) => format!(" ({n})"),
                _ => String::new(),
            };
            let _ = writeln!(
                s,
                "- {} — {} {}{}",
                fmt_time(a.created_at),
                a.actor_name,
                label(&ACTION, a.action),
                detail
            );
        }
        let _ = writeln!(s);
    }

    s
}
