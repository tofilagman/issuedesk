-- Customer role (2) + admin-managed user groups. Customers only see issues
-- reported by themselves or by users sharing at least one group with them.

ALTER TABLE users DROP CONSTRAINT users_role_chk;
ALTER TABLE users ADD CONSTRAINT users_role_chk CHECK (role BETWEEN 0 AND 2);
-- role: 0=member, 1=admin, 2=customer

CREATE TABLE groups (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name        text NOT NULL UNIQUE,
    description text,
    created_by  uuid NOT NULL REFERENCES users(id),
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE group_members (
    group_id uuid NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id  uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    added_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (group_id, user_id)
);

-- The visibility predicate self-joins group_members (viewer -> group -> reporter):
-- this index serves the viewer-side lookup, the PK serves the reporter side.
CREATE INDEX idx_group_members_user ON group_members(user_id);
