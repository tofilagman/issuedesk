-- Add "Implemented" as status 2; shift InReview 2→3 and Done 3→4.
ALTER TABLE issues DROP CONSTRAINT issues_status_chk;

-- Shift Done (3→4) first, then InReview (2→3) to avoid collisions.
UPDATE issues SET status = 4 WHERE status = 3;
UPDATE issues SET status = 3 WHERE status = 2;

ALTER TABLE issues ADD CONSTRAINT issues_status_chk CHECK (status BETWEEN 0 AND 4);

-- Keep activity_log history consistent.
UPDATE activity_log SET new_value = '4' WHERE action = 1 AND new_value = '3';
UPDATE activity_log SET old_value = '4' WHERE action = 1 AND old_value = '3';
UPDATE activity_log SET new_value = '3' WHERE action = 1 AND new_value = '2';
UPDATE activity_log SET old_value = '3' WHERE action = 1 AND old_value = '2';
