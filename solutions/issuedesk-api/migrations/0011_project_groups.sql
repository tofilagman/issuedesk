-- Link groups to projects: every member of a linked group gets project access
-- without needing an individual project_members row.

CREATE TABLE project_groups (
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    group_id   uuid NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    added_at   timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, group_id)
);

-- Reverse lookup: "which projects does this group unlock?" (list_visible).
CREATE INDEX idx_project_groups_group ON project_groups(group_id);
