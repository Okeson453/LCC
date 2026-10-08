-- 0022_identity_member_settings_alignment.sql
-- Bring `lcc.members` up to the member-account shape that identity-svc reads.
--
-- WHY THIS MIGRATION EXISTS
-- -------------------------
-- identity-svc selects five columns off `lcc.members` that migration 0003
-- never created, so every identity read failed at runtime with
-- `column ... does not exist` while the statement still PREPAREd cleanly.
--
-- Three independent artefacts describe the member account and all three agree
-- with each other, disagreeing only with the 0003 table:
--
--   1. Backend Design Concept §11.1 `member_account` (explicit CREATE TABLE
--      with the columns, types, CHECK constraint and defaults);
--   2. Contract/openapi/lcc-api-canonical.yaml -> `MemberSettings`
--      (active_goal_mode with the same three-value enum);
--   3. identity-svc's own SQL, which reads and writes them.
--
-- This is the same situation 0017 resolved for `lcc.opportunities` and 0020 for
-- `lcc.contacts`: the table is the stale artefact, not the consumers.
--
-- WHY THESE ARE COLUMNS AND NOT DERIVED VIEWS
-- --------------------------------------------
-- It is tempting to derive the restriction fields from `lcc.restrictions`
-- (migration 0010), which does exist. That would be wrong.
--
-- The design is explicit that restriction is a latched state with manual
-- clearance and no automatic expiry:
--
--   "On any match -> immediate circuit-break: all queues pause for that
--    account, member_account.is_restricted=true, restricted_since=now(),
--    compliance.restriction_detected event emitted. Manual review required;
--    no auto-expiry."  (Design §6 Non-Negotiable, repeated at §18)
--
-- A derivation from `restrictions` rows would re-clear the moment a row was
-- archived or its `cleared_at` was set, which is exactly the auto-expiry the
-- design forbids. `is_restricted` is a flag the compliance-governor owns; it
-- must be stored where it can be set and cleared deliberately.
--
-- `active_goal_mode` and `warmup_started_at` are member settings, not derived
-- from anything.
--
-- `oauth_*` columns from the design are deliberately NOT added: migration 0003
-- already put tokens in their own `lcc.oauth_tokens` table, which is the
-- better design (one row per token, its own RLS and expiry) and is what the
-- services actually read. The 0003 split is an improvement on §11.1 and is
-- kept.

BEGIN;

-- The three-value goal mode. Kept as a CHECK rather than a new enum type so
-- this migration adds no type to the shared namespace; the contract declares
-- exactly these three values.
ALTER TABLE lcc.members
    ADD COLUMN IF NOT EXISTS active_goal_mode TEXT NOT NULL DEFAULT 'hybrid'
        CHECK (active_goal_mode IN ('job_hunting', 'client_acquisition', 'hybrid')),

    -- Latched compliance state. See the header for why this is stored and not
    -- derived from lcc.restrictions.
    ADD COLUMN IF NOT EXISTS is_restricted BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS restricted_since TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS restricted_reason TEXT,
    ADD COLUMN IF NOT EXISTS restricted_cleared_at TIMESTAMPTZ,

    -- When this member's sending warm-up window opened. Design §11.1
    -- NOT NULL DEFAULT now(): every existing row gets a value, so the column
    -- can be NOT NULL and the service never has to invent one.
    ADD COLUMN IF NOT EXISTS warmup_started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Optimistic concurrency, matching the convention 0017/0020 established.
    ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- The compliance dashboard lists restricted members on every request; without
-- this it is a sequential scan of the whole member table.
CREATE INDEX IF NOT EXISTS idx_members_restricted
    ON lcc.members (is_restricted, restricted_since DESC)
    WHERE is_restricted;

-- Backfill the latch for any member that already has an open restriction row,
-- so an environment upgraded into this migration does not silently report
-- "not restricted" for a member the governor has already flagged. Only open
-- rows (cleared_at IS NULL) count, which is the design's manual-clear
-- semantics: a cleared restriction does not re-restrict the member.
UPDATE lcc.members m
   SET is_restricted = TRUE,
       restricted_since = COALESCE(m.restricted_since, r.detected_at)
  FROM lcc.restrictions r
 WHERE r.member_id = m.id
   AND r.cleared_at IS NULL
   AND m.is_restricted = FALSE;

COMMIT;
