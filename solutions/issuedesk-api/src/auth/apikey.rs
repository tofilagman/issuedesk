//! API key minting and hashing.
//!
//! A key is a high-entropy random secret of the form `idk_<48 hex chars>`. We
//! store only its SHA-256 hash (the secret is high-entropy, so a fast,
//! unsalted hash is appropriate and gives an O(1) lookup on every request).
//! The plaintext secret is shown to the user exactly once, at creation.

use sha2::{Digest, Sha256};
use uuid::Uuid;

const PREFIX: &str = "idk_";

/// A freshly minted key: the plaintext `secret` (returned once), its `hash`
/// (stored), and a short non-secret `prefix` for display.
pub struct NewApiKey {
    pub secret: String,
    pub hash: String,
    pub prefix: String,
}

/// Mint a new API key. Two v4 UUIDs (~244 bits of entropy) provide the random
/// body, hex-encoded — no extra RNG dependency beyond `uuid`.
pub fn generate() -> NewApiKey {
    let body = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let secret = format!("{PREFIX}{body}");
    let hash = hash(&secret);
    // Display prefix: scheme + first 6 chars of the body.
    let prefix = format!("{PREFIX}{}", &body[..6]);
    NewApiKey { secret, hash, prefix }
}

/// SHA-256 hex digest of a key secret.
pub fn hash(secret: &str) -> String {
    let digest = Sha256::digest(secret.as_bytes());
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// True if a string looks like one of our API keys (cheap pre-check before a
/// DB lookup, lets us tell API keys apart from JWTs in the Authorization header).
pub fn looks_like_key(s: &str) -> bool {
    s.starts_with(PREFIX)
}
