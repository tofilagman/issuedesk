-- Comments and labels.

CREATE TABLE comments (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id   uuid NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    author_id  uuid NOT NULL REFERENCES users(id),
    body       text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_comments_issue ON comments(issue_id, created_at);

CREATE TABLE labels (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name       text NOT NULL,
    color      text NOT NULL,
    UNIQUE (project_id, name)
);

CREATE TABLE issue_labels (
    issue_id uuid NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    label_id uuid NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (issue_id, label_id)
);

CREATE INDEX idx_issue_labels_label ON issue_labels(label_id);
