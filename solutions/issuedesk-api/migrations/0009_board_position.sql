-- Manual board ordering: a fractional rank per issue, ordered within its
-- (project, status) column. Drag-drop reorder sets a midpoint between neighbors,
-- so inserts don't renumber the whole column.

ALTER TABLE issues ADD COLUMN board_position double precision NOT NULL DEFAULT 0;

-- Seed positions from the current board order (newest number on top), so the
-- board looks unchanged until someone drags a card.
WITH ranked AS (
    SELECT id,
           row_number() OVER (PARTITION BY project_id, status ORDER BY number DESC) AS rn
    FROM issues
)
UPDATE issues i
   SET board_position = r.rn
  FROM ranked r
 WHERE r.id = i.id;

CREATE INDEX idx_issues_board_order ON issues(project_id, status, board_position);
