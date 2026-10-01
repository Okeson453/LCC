-- 0015_sequence_step_governance.sql
-- Audit F-AUDIT-12: the sequence-step scheduler recorded sends without ever
-- consulting the Compliance Governor, and had no way to record that a step
-- was denied. It also had no column to carry the failure reason, so any
-- delivery error was silently swallowed.
--
-- Adds the columns required for fail-closed governor accounting:
--   - denial_count : consecutive governor denials, used to move a
--                    permanently-unpermittable step to `blocked` instead of
--                    retrying forever.
--   - last_error   : the last governor / dispatch failure reason, surfaced to
--                    the user rather than discarded.
--   - step_status  : the original status string is kept for back-compat; the
--                    new 'pending_evaluation' value is what the scheduler
--                    writes while awaiting a governor decision.
--
-- The `status` column remains TEXT (not an enum) so the new intermediate
-- state does not require an enum migration and so an unknown value from an
-- older/newer worker cannot abort an insert.

BEGIN;

ALTER TABLE lcc.sequence_steps
    ADD COLUMN IF NOT EXISTS denial_count INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_error TEXT,
    ADD COLUMN IF NOT EXISTS permit_token TEXT;

-- Supports the scheduler's hot query: due, not-yet-evaluated steps.
CREATE INDEX IF NOT EXISTS idx_sequence_steps_due
    ON lcc.sequence_steps (scheduled_at)
    WHERE status IN ('pending', 'pending_evaluation');

COMMIT;
