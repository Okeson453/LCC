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

-- F-AUDIT-42 (verified against a real PostgreSQL 15 run of the full migration
-- set): the header above states "0018 added `tier` / `version` /
-- `decided_reason`", but 0018 aborted before reaching its approvals block, so
-- none of those columns were ever created. The real `lcc.approvals` table had
-- only: id, member_id, resource_type, resource_id, requested_action, decision,
-- decided_by, decided_at, expires_at, decision_rationale, created_at.
--
-- Both the canonical contract (`components.schemas.Approval`) and
-- approval-svc's `ApprovalRow` read/write `tier`, `rule_version`,
-- `requested_by`, `decided_reason` and `version`. Every approval-svc query
-- would therefore have failed at runtime with "column tier does not exist",
-- taking the two-reviewer gate, the tiered decision rules and optimistic
-- locking (`WHERE ... AND version = $3`) entirely offline.
--
-- The columns are added here, idempotently, aligned to the contract:
--   tier           INT  tier 1..5 (contract: minimum 1, maximum 5)
--   rule_version   TEXT version of the rule set that produced the decision
--   requested_by   TEXT 'system:<service>' per the contract description
--   decided_reason TEXT  free-text rationale
--   version        INT  optimistic-concurrency counter
-- `decided_by` (TEXT) is already present from 0011 and is kept as-is.
ALTER TABLE lcc.approvals
    ADD COLUMN IF NOT EXISTS tier INT NOT NULL DEFAULT 1
        CONSTRAINT approvals_tier_range CHECK (tier BETWEEN 1 AND 5),
    ADD COLUMN IF NOT EXISTS rule_version TEXT,
    ADD COLUMN IF NOT EXISTS requested_by TEXT NOT NULL DEFAULT 'system:unknown',
    ADD COLUMN IF NOT EXISTS decided_reason TEXT,
    ADD COLUMN IF NOT EXISTS version INT NOT NULL DEFAULT 1;

COMMENT ON COLUMN lcc.approvals.tier IS
  'Risk tier 1-5. Tier >= 3 requires two distinct reviewers before approval.';
COMMENT ON COLUMN lcc.approvals.version IS
  'Optimistic-concurrency counter; updated on every decision transition.';

COMMENT ON COLUMN lcc.approvals.reviewer_ids IS
  'Distinct reviewers who have signed this approval. Tier>=3 approvals require '
  'at least two entries before they may transition to approved.';

-- Partial index supporting "which tier>=3 approvals are still short of a
-- second signature".
--
-- F-AUDIT-40 (verified against a real PostgreSQL 15 run): the predicate used
-- `array_length(reviewer_ids, 1) < 2`, but index predicates must be IMMUTABLE.
-- `array_length` is STABLE, not IMMUTABLE, so the statement was rejected.
-- `cardinality()` is the immutable, semantically equivalent form, and it
-- handles the NULL case the same way (NULL -> predicate is not true).
CREATE INDEX IF NOT EXISTS idx_approvals_pending_second_reviewer
  ON lcc.approvals (member_id, tier)
  WHERE decision = 'pending' AND tier >= 3 AND cardinality(reviewer_ids) < 2;
