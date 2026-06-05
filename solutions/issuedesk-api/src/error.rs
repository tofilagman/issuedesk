use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("payload too large")]
    PayloadTooLarge,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m.clone()),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "unauthorized", m.clone()),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, "forbidden", m.clone()),
            AppError::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, "validation", m.clone()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, "conflict", m.clone()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m.clone()),
            AppError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "uploaded file exceeds the allowed size".to_string(),
            ),
            AppError::Database(e) => {
                // Special-case the common, expected sqlx errors before falling
                // back to an opaque 500.
                match e {
                    sqlx::Error::RowNotFound => {
                        (StatusCode::NOT_FOUND, "not_found", "resource not found".to_string())
                    }
                    sqlx::Error::Database(db) if db.code().as_deref() == Some("23505") => {
                        (StatusCode::CONFLICT, "conflict", "resource already exists".to_string())
                    }
                    _ => {
                        tracing::error!(error = ?e, "database error");
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "internal",
                            "internal server error".to_string(),
                        )
                    }
                }
            }
            AppError::Internal(e) => {
                tracing::error!(error = ?e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "internal server error".to_string(),
                )
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();
        let body = Json(json!({ "error": code, "message": message }));
        (status, body).into_response()
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(e: validator::ValidationErrors) -> Self {
        // Flatten the first message per field into a single readable string.
        let mut msgs = Vec::new();
        for (field, errs) in e.field_errors() {
            let detail = errs
                .iter()
                .filter_map(|err| err.message.as_ref().map(|m| m.to_string()))
                .next()
                .unwrap_or_else(|| "invalid value".to_string());
            msgs.push(format!("{field}: {detail}"));
        }
        AppError::Validation(msgs.join("; "))
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized("invalid or expired token".to_string())
    }
}
