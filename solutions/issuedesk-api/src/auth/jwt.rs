use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::Result;

/// JWT claims. Memberships are intentionally NOT embedded (they change) — project
/// authorization is resolved per-request from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    #[serde(rename = "userName")]
    pub user_name: String,
    pub role: i16,
    pub exp: i64,
    pub iat: i64,
}

pub fn issue_token(
    secret: &str,
    user_id: Uuid,
    user_name: &str,
    role: i16,
    ttl_hours: i64,
) -> Result<String> {
    let now = OffsetDateTime::now_utc();
    let claims = Claims {
        sub: user_id,
        user_name: user_name.to_string(),
        role,
        iat: now.unix_timestamp(),
        exp: (now + time::Duration::hours(ttl_hours)).unix_timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn verify_token(secret: &str, token: &str) -> Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}
