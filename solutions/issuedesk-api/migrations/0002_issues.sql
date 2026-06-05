-- Issues: belong to a project, carry a per-project number (KEY-number).

CREATE TABLE issues (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id  uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    number      bigint NOT NULL,
    title       text NOT NULL,
    description text,
    type        smallint NOT NULL DEFAULT 1,   -- 0 bug,1 task,2 story,3 epic
    status      smallint NOT NULL DEFAULT 0,   -- 0 todo,1 in_progress,2 in_review,3 done
    priority    smallint NOT NULL DEFAULT 1,   -- 0 low,1 medium,2 high,3 urgent
    assignee_id uuid REFERENCES users(id),
    reporter_id uuid NOT NULL REFERENCES users(id),
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    UNIQUE (project_id, number),
    CONSTRAINT issues_type_chk     CHECK (type BETWEEN 0 AND 3),
    CONSTRAINT issues_status_chk   CHECK (status BETWEEN 0 AND 3),
    CONSTRAINT issues_priority_chk CHECK (priority BETWEEN 0 AND 3)
);

CREATE INDEX idx_issues_project_status   ON issues(project_id, status);
CREATE INDEX idx_issues_project_assignee ON issues(project_id, assignee_id);
CREATE INDEX idx_issues_project_number   ON issues(project_id, number);
