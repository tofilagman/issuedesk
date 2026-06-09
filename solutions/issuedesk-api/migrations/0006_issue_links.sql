-- Issue links: directed relationships between issues (relates / blocks / duplicates).
-- Stored in a CANONICAL direction so each relationship is one row:
--   0 = relates to     (symmetric; source/target stored in id order)
--   1 = blocks         (source blocks target)
--   3 = duplicates     (source duplicates target)
-- The "is blocked by" / "is duplicated by" directions are derived at read time
-- by looking at which side the viewing issue is on (see db::links).

CREATE TABLE issue_links (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source_issue_id uuid NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    target_issue_id uuid NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    link_type       smallint NOT NULL,
    created_by      uuid NOT NULL REFERENCES users(id),
    created_at      timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT issue_links_no_self  CHECK (source_issue_id <> target_issue_id),
    CONSTRAINT issue_links_type_chk CHECK (link_type IN (0, 1, 3)),
    UNIQUE (source_issue_id, target_issue_id, link_type)
);

CREATE INDEX idx_issue_links_source ON issue_links(source_issue_id);
CREATE INDEX idx_issue_links_target ON issue_links(target_issue_id);
