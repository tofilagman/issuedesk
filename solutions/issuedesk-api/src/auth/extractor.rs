use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{
    auth::jwt::Claims,
    error::AppError,
    models::enums::Role,
};

/// The authenticated caller, extracted from claims placed by `require_auth`.
/// Handlers take `user: AuthUser` to access the current identity.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub claims: Claims,
}

impl AuthUser {
    pub fn id(&self) -> uuid::Uuid {
        self.claims.sub
    }
    pub fn is_admin(&self) -> bool {
        self.claims.role == Role::Admin.as_i16()
    }
    /// Returns `Forbidden` unless the caller is a global admin.
    pub fn require_admin(&self) -> Result<(), AppError> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(AppError::Forbidden("admin role required".to_string()))
        }
    }
    pub fn is_customer(&self) -> bool {
        self.claims.role == Role::Customer.as_i16()
    }
    /// Returns `Forbidden` when the caller is a customer account (write
    /// operations customers may not perform).
    pub fn require_not_customer(&self) -> Result<(), AppError> {
        if self.is_customer() {
            Err(AppError::Forbidden(
                "not permitted for customer accounts".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("not authenticated".to_string()))?;
        Ok(AuthUser { claims })
    }
}
