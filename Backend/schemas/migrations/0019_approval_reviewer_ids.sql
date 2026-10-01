-- 0019_approval_reviewer_ids.sql
-- The two-reviewer gate for tier>=3 approvals needs to know *which* members
-- have already signed. `lcc.approvals` (0011) carries a single `decided_by`
-- TEXT column and 0018 added `tier` / `version` / `decided_reason`, but there
-- was no column holding the set of reviewers.
--
-- `approval-svc`'s decide path already wrote to `reviewer_ids`:
--     reviewer_ids = CASE WHEN $6::uuid = ANY(reviewer_ids) ...
-- so the statement failed at runtime with "column reviewer_ids does not exist",
-- and the two-reviewer gate could never be satisfied.
--
-- The column is an append-only UUID array: the CASE expression above keeps it
-- set-like (a repeat signature is a no-op), so a reviewer appears at most once
-- and the gate is a simple cardinality check.

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc'
                   AND table_name = 'approvals'
                   AND column_name = 'reviewer_ids') THEN
    ALTER TABLE lcc.approvals
      ADD COLUMN reviewer_ids UUID[] NOT NULL DEFAULT '{}';
  END IF;
END$$;

COMMENT ON COLUMN lcc.approvals.reviewer_ids IS
  'Distinct reviewers who have signed this approval. Tier>=3 approvals require '
  'at least two entries before they may transition to approved.';

-- Partial index supporting "which tier>=3 approvals are still short of a
-- second signature".
CREATE INDEX IF NOT EXISTS idx_approvals_pending_second_reviewer
  ON lcc.approvals (member_id, tier)
  WHERE decision = 'pending' AND tier >= 3 AND array_length(reviewer_ids, 1) < 2;
