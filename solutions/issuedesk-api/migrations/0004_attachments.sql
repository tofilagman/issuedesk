-- File attachments. Binary lives on disk under UPLOAD_DIR; the row is metadata.

CREATE TABLE attachments (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id    uuid NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    filename    text NOT NULL,        -- original (display) name
    stored_path text NOT NULL,        -- path relative to UPLOAD_DIR
    size_bytes  bigint NOT NULL,
    mime_type   text NOT NULL,
    uploaded_by uuid NOT NULL REFERENCES users(id),
    created_at  timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_attachments_issue ON attachments(issue_id);
