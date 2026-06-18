-- API keys: long-lived, read-only credentials for non-browser clients
-- (e.g. relaying a ticket to Claude). The secret is shown once at creation;
-- only its SHA-256 hash is stored. `prefix` is a short, non-secret label for
-- the UI to identify a key.

CREATE TABLE api_keys (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name         text NOT NULL,
    key_hash     text NOT NULL UNIQUE,   -- sha256 hex of the full secret
    prefix       text NOT NULL,          -- e.g. "idk_a1b2c3" (display only)
    created_by   uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_used_at timestamptz,
    created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_api_keys_created_by ON api_keys(created_by);
