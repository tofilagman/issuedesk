use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{auth::jwt, error::AppError, state::AppState};

/// Guards `/api/**`. Validates the JWT Bearer token and enforces the
/// `Sec-Fetch-Site` browser-lock, then stashes the decoded claims in request
/// extensions for the `AuthUser` extractor to pick up.
///
/// Note for curl/Postman: requests must include
/// `-H 'Sec-Fetch-Site: same-origin'`.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // --- Browser-lock (Sec-Fetch-Site) ---
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
