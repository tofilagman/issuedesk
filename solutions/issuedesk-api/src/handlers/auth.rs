use axum::{extract::State, Json};
use validator::Validate;

use crate::{
    auth::{jwt, password},
    db,
    dto::{SignInRequest, TokenResponse},
    error::{AppError, Result},
    models::UserDto,
    state::AppState,
};

pub async fn sign_in(
    State(state): State<AppState>,
    Json(req): Json<SignInRequest>,
) -> Result<Json<TokenResponse>> {
    req.validate()?;

    let user = db::users::find_by_username(&state.pool, &req.user_name)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

    if !user.is_active {
        return Err(AppError::Forbidden("account is disabled".into()));
    }

    password::verify_password(&req.password, &user.password_hash)?;

    let token = jwt::issue_token(
        &state.config.jwt_secret,
        user.id,
        &user.user_name,
        user.role,
        state.config.jwt_ttl_hours,
    )?;

    Ok(Json(TokenResponse {
        token,
        user: UserDto::from(user),
    }))
}
