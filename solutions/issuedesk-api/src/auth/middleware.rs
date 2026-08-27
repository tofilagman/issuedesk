use axum::{
    extract::{Request, State},
    http::Method,
    middleware::Next,
    response::Response,
};
use time::OffsetDateTime;

use crate::{
    auth::{apikey, jwt, jwt::Claims},
    db,
    error::AppError,
    models::enums::Role,
    state::AppState,
};

/// Guards `/api/**`. Two ways to authenticate:
///
/// * **JWT Bearer** (the browser/SPA): validates the token and enforces the
///   `Sec-Fetch-Site` browser-lock.
/// * **API key** (non-browser clients, e.g. relaying a ticket to Claude):
///   supplied via `X-API-Key` or `Authorization: Bearer idk_…`. API keys are
///   read-only except for managing comments and attachments (`POST`/`PATCH`/
///   `DELETE` on a `…/comments` or `…/attachments` path), and are exempt from
///   the browser-lock, since curl/agents
///   do not send `Sec-Fetch-Site`. A
///   key acts on behalf of the user who created it (its `created_by`), so a
///   comment it posts is attributed to that real user.
///
/// Either path stashes decoded `Claims` in request extensions for the
/// `AuthUser` extractor.
///
/// Note for curl/Postman using a JWT: requests must include
/// `-H 'Sec-Fetch-Site: same-origin'`. API-key requests do not.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // --- API key path (X-API-Key header, or a Bearer token shaped like a key) ---
    if let Some(secret) = extract_api_key(&req) {
        // Read-only, with three families of exceptions: managing comments,
        // attachments, and a ticket's description. A key may post/edit/delete
        // comments (`…/comments`, `…/comments/{id}`), upload/delete attachments
        // (`…/attachments`, `…/attachments/{id}`) — so a relay client can comment
        // back with a file — and replace a ticket's description
        // (`/api/tickets/{slug}/description`). Note the last one is a dedicated
        // sub-path precisely so it can be allowed here without opening up
        // `PATCH /api/tickets/{slug}` (status, assignee, type…).
        // Everything else must be a safe (GET) request.
        let path = req.uri().path();
        let is_relay_write = matches!(*req.method(), Method::POST | Method::PATCH | Method::DELETE)
            && (path.ends_with("/comments")
                || path.contains("/comments/")
                || path.ends_with("/attachments")
                || path.contains("/attachments/")
                || path.ends_with("/description"));
        if req.method() != Method::GET && !is_relay_write {
            return Err(AppError::Forbidden(
                "API keys are read-only (except managing comments and attachments)".to_string(),
            ));
        }
        let key = db::api_keys::find_by_hash(&state.pool, &apikey::hash(&secret))
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid API key".to_string()))?;

        // Best-effort usage stamp; never block the request on it.
        let _ = db::api_keys::touch(&state.pool, key.id).await;

        // Synthetic principal: admin role so it can read across all projects.
        // `sub` is the key's creator, so a comment it posts is attributed to a
        // real user and satisfies the comments.author_id foreign key.
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let claims = Claims {
            sub: key.created_by,
            user_name: format!("apikey:{}", key.name),
            role: Role::Admin.as_i16(),
            iat: now,
            exp: now,
        };
        req.extensions_mut().insert(claims);
        return Ok(next.run(req).await);
    }

    // --- Browser-lock (Sec-Fetch-Site) — JWT path only ---
    let allowed = req
        .headers()
        .get("sec-fetch-site")
        .and_then(|v| v.to_str().ok())
        .map(|v| matches!(v, "same-origin" | "same-site" | "none"))
        .unwrap_or(false);
    if !allowed {
        return Err(AppError::Forbidden(
            "request rejected by browser-lock (missing/invalid Sec-Fetch-Site)".to_string(),
        ));
    }

    // --- Bearer token ---
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".to_string()))?;

    let claims = jwt::verify_token(&state.config.jwt_secret, token)?;
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// Pull an API key secret from the request: a dedicated `X-API-Key` header, or
/// an `Authorization: Bearer` token that is shaped like one of our keys.
fn extract_api_key(req: &Request) -> Option<String> {
    if let Some(v) = req.headers().get("x-api-key").and_then(|v| v.to_str().ok()) {
        let v = v.trim();
        if !v.is_empty() {
            return Some(v.to_string());
        }
    }
    let bearer = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))?;
    apikey::looks_like_key(bearer).then(|| bearer.to_string())
}
