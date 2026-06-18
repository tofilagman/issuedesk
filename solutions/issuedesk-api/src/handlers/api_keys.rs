use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::{apikey, AuthUser},
    db,
    dto::{ApiKeyCreated, CreateApiKeyRequest},
    error::Result,
    models::ApiKeyDto,
    state::AppState,
};

pub async fn list(State(state): State<AppState>, user: AuthUser) -> Result<Json<Vec<ApiKeyDto>>> {
    user.require_admin()?;
    let rows = db::api_keys::list(&state.pool).await?;
    Ok(Json(rows.into_iter().map(ApiKeyDto::from).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiKeyCreated>> {
    user.require_admin()?;
    req.validate()?;
    let minted = apikey::generate();
    let row = db::api_keys::create(
        &state.pool,
        &req.name,
        &minted.hash,
        &minted.prefix,
        user.id(),
    )
    .await?;
    Ok(Json(ApiKeyCreated {
        secret: minted.secret,
        key: ApiKeyDto::from(row),
    }))
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    user.require_admin()?;
    db::api_keys::delete(&state.pool, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
