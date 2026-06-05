-- Append-only per-issue audit trail.

CREATE TABLE activity_log (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id   uuid NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    actor_id   uuid NOT NULL REFERENCES users(id),
    action     smallint NOT NULL,   -- see models::enums::ActivityAction
    field      text,
    old_value  text,
    new_value  text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_activity_issue ON activity_log(issue_id, created_at DESC);
