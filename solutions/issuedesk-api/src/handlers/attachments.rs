use std::path::{Path as FsPath, PathBuf};

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{AppError, Result},
    models::AttachmentRow,
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
) -> Result<Json<Vec<AttachmentRow>>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;
    let rows = db::attachments::list(&state.pool, issue_id).await?;
    Ok(Json(rows))
}

pub async fn upload(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<AttachmentRow>> {
    let project_id = db::issues::project_of(&state.pool, issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;

    // Take the first file field.
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("invalid multipart: {e}")))?
        .ok_or_else(|| AppError::BadRequest("no file field in request".into()))?;

    let original_name = field
        .file_name()
        .map(sanitize_filename)
        .unwrap_or_else(|| "upload.bin".to_string());
    let mime_type = field
        .content_type()
        .map(|s| s.to_string())
        .or_else(|| {
            mime_guess::from_path(&original_name)
                .first()
                .map(|m| m.to_string())
        })
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Server-generated storage path (never trust client paths).
    let ext = FsPath::new(&original_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let rel_path = format!("{project_id}/{issue_id}/{}.{ext}", Uuid::new_v4());
    let abs_path = PathBuf::from(&state.config.upload_dir).join(&rel_path);
    if let Some(parent) = abs_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("mkdir failed: {e}")))?;
    }

    // Stream the field to disk, enforcing the size limit as we go.
    let max = state.config.max_upload_bytes;
    let mut file = tokio::fs::File::create(&abs_path)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("create file failed: {e}")))?;
    let mut written: usize = 0;
    let mut field = field;
    loop {
        let chunk = field
            .chunk()
            .await
            .map_err(|e| AppError::BadRequest(format!("upload read error: {e}")))?;
        let Some(bytes) = chunk else { break };
        written += bytes.len();
        if written > max {
            drop(file);
            let _ = tokio::fs::remove_file(&abs_path).await;
            return Err(AppError::PayloadTooLarge);
        }
        file.write_all(&bytes)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("write failed: {e}")))?;
    }
    file.flush()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("flush failed: {e}")))?;

    let row = db::attachments::create(
        &state.pool,
        issue_id,
        &original_name,
        &rel_path,
        written as i64,
        &mime_type,
        user.id(),
    )
    .await?;
    Ok(Json(row))
}

pub async fn download(
    State(state): State<AppState>,
    user: AuthUser,
    Path(attachment_id): Path<Uuid>,
) -> Result<Response> {
    let att = db::attachments::get(&state.pool, attachment_id).await?;
    let project_id = db::issues::project_of(&state.pool, att.issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;

    let abs_path = PathBuf::from(&state.config.upload_dir).join(&att.stored_path);
    let file = tokio::fs::File::open(&abs_path)
        .await
        .map_err(|_| AppError::NotFound("file missing on disk".into()))?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let content_disp = format!("attachment; filename=\"{}\"", att.filename.replace('"', ""));
    let resp = (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, att.mime_type),
            (header::CONTENT_DISPOSITION, content_disp),
        ],
        body,
    )
        .into_response();
    Ok(resp)
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(attachment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let att = db::attachments::get(&state.pool, attachment_id).await?;
    let project_id = db::issues::project_of(&state.pool, att.issue_id).await?;
    db::authorize_project(&state.pool, &user, project_id).await?;

    let row = db::attachments::delete(&state.pool, attachment_id, user.id()).await?;
    let abs_path = PathBuf::from(&state.config.upload_dir).join(&row.stored_path);
    let _ = tokio::fs::remove_file(&abs_path).await; // best-effort
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Strip any directory components and keep a safe basename.
fn sanitize_filename(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let cleaned: String = base
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>();
    if cleaned.is_empty() {
        "upload.bin".to_string()
    } else {
        cleaned
    }
}
