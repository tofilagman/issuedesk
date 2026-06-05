use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::error::{AppError, Result};

/// Hash a plaintext password into an argon2 PHC string.
pub fn hash_password(plain: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("password hash failed: {e}")))?;
    Ok(hash.to_string())
}

/// Verify a plaintext password against a stored PHC hash. Returns `Ok(())` on
/// match, `Unauthorized` on mismatch.
pub fn verify_password(plain: &str, stored_hash: &str) -> Result<()> {
    let parsed = PasswordHash::new(stored_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("bad stored hash: {e}")))?;
    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized("invalid credentials".to_string()))
}
