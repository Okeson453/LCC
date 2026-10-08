-- 0018_sequence_steps_kb_alignment.sql
-- Phase B — DB foundation gap closure per contract_audit.
--
-- Aligns the sequence_steps table with the canonical contract fields
-- (sequence_step.kind, idempotency_key uniqueness), and adds the missing
-- `updated_at`/touch trigger that some tables lack. Also adds the
-- canonical compliance_audit_chain column expected by design §17.

BEGIN;

-- -----------------------------------------------------------------------------
-- sequence_steps — add missing columns the canonical contract requires.
-- 0015 added denial_count, last_error; this adds the rest.
-- -----------------------------------------------------------------------------
-- F-AUDIT-38 (verified against a real PostgreSQL 15 run): the migration created
-- a unique index on `idempotency_key` but never added the column to the table,
-- so the index statement failed with
--     ERROR: column "idempotency_key" does not exist
-- The intent — stated in this migration's own header, "sequence_step.kind,
-- idempotency_key uniqueness" — is a dedup key, so the column is added here.
-- Uniqueness is enforced per (sequence_id, key) rather than globally: a
-- client-supplied key only has to be unique within the sequence it belongs to.
ALTER TABLE lcc.sequence_steps
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS rendered_body_hash TEXT,   -- SHA-256 of body for idempotency dedup
    ADD COLUMN IF NOT EXISTS contact_id UUID,           -- denormalised for fast filters; FK enforced in code
    ADD COLUMN IF NOT EXISTS idempotency_key TEXT;

CREATE INDEX IF NOT EXISTS idx_sequence_steps_contact
    ON lcc.sequence_steps (contact_id)
    WHERE contact_id IS NOT NULL;

-- Idempotency: when an idempotency_key is present it must be unique per sequence.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_indexes
    WHERE schemaname = 'lcc'
      AND tablename = 'sequence_steps'
      AND indexname = 'uniq_sequence_steps_idem_key'
  ) THEN
    CREATE UNIQUE INDEX uniq_sequence_steps_idem_key
      ON lcc.sequence_steps (sequence_id, idempotency_key)
      WHERE idempotency_key IS NOT NULL;
  END IF;
END$$;

-- Touch trigger
DROP TRIGGER IF EXISTS sequence_steps_touch ON lcc.sequence_steps;
CREATE TRIGGER sequence_steps_touch BEFORE UPDATE ON lcc.sequence_steps
    FOR EACH ROW EXECUTE FUNCTION lcc.touch_updated_at();

-- -----------------------------------------------------------------------------
-- content_items — add the canonical columns the design §7.1 calls for but
-- the existing migrations don't define.
-- -----------------------------------------------------------------------------
ALTER TABLE lcc.content_items
    ADD COLUMN IF NOT EXISTS quality_loop_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS metrics JSONB NOT NULL DEFAULT '{}'::JSONB,
    ADD COLUMN IF NOT EXISTS idempotency_key TEXT,
    ADD COLUMN IF NOT EXISTS expected_version BIGINT;       -- for optimistic-concurrency

-- The canonical contract requires `status` enum values: idea, drafting,
-- quality_check, pending_approval, approved, rejected, scheduled, publishing,
-- published, publish_failed, blocked, archived.
--
-- The existing lcc.content_state ENUM is:
--   'drafting', 'quality_check', 'pending_approval',
--   'scheduled', 'publishing', 'published', 'failed',
--   'rejected', 'archived'
--
-- It is missing: 'idea', 'approved', 'publish_failed', 'blocked'.
-- Add them via ALTER TYPE. Each is idempotent.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_enum e
                 JOIN pg_type t ON e.enumtypid = t.oid
                 WHERE t.typname = 'content_state' AND e.enumlabel = 'idea') THEN
    ALTER TYPE lcc.content_state ADD VALUE 'idea';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_enum e
                 JOIN pg_type t ON e.enumtypid = t.oid
                 WHERE t.typname = 'content_state' AND e.enumlabel = 'approved') THEN
    ALTER TYPE lcc.content_state ADD VALUE 'approved';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_enum e
                 JOIN pg_type t ON e.enumtypid = t.oid
                 WHERE t.typname = 'content_state' AND e.enumlabel = 'publish_failed') THEN
    ALTER TYPE lcc.content_state ADD VALUE 'publish_failed';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_enum e
                 JOIN pg_type t ON e.enumtypid = t.oid
                 WHERE t.typname = 'content_state' AND e.enumlabel = 'blocked') THEN
    ALTER TYPE lcc.content_state ADD VALUE 'blocked';
  END IF;
END$$;

-- Updated_at trigger if missing.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'content_items_touch') THEN
    CREATE TRIGGER content_items_touch BEFORE UPDATE ON lcc.content_items
        FOR EACH ROW EXECUTE FUNCTION lcc.touch_updated_at();
  END IF;
END$$;

-- Unique idempotency key (member-scoped).
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_indexes
    WHERE schemaname = 'lcc' AND tablename = 'content_items' AND indexname = 'uniq_content_items_idem'
  ) THEN
    CREATE UNIQUE INDEX uniq_content_items_idem
      ON lcc.content_items (member_id, idempotency_key)
      WHERE idempotency_key IS NOT NULL;
  END IF;
END$$;

-- -----------------------------------------------------------------------------
-- approvals — add `tier` column if missing (design §18).
-- 0011 already declared resource_type / resource_id / decision; tier was
-- only inferred via `requested_action->>'tier'` reads.
-- -----------------------------------------------------------------------------
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc'
                   AND table_name = 'approvals'
                   AND column_name = 'tier') THEN
    ALTER TABLE lcc.approvals
      ADD COLUMN tier SMALLINT NOT NULL DEFAULT 1
        CHECK (tier BETWEEN 1 AND 5);
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc' AND table_name = 'approvals' AND column_name = 'rule_version') THEN
    ALTER TABLE lcc.approvals
      ADD COLUMN rule_version TEXT NOT NULL DEFAULT 'unknown';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc' AND table_name = 'approvals' AND column_name = 'requested_by') THEN
    ALTER TABLE lcc.approvals
      ADD COLUMN requested_by TEXT NOT NULL DEFAULT 'system:unknown';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc' AND table_name = 'approvals' AND column_name = 'version') THEN
    ALTER TABLE lcc.approvals
      ADD COLUMN version BIGINT NOT NULL DEFAULT 1;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc' AND table_name = 'approvals' AND column_name = 'decided_reason') THEN
    ALTER TABLE lcc.approvals
      ADD COLUMN decided_reason TEXT;
  END IF;
END$$;

CREATE INDEX IF NOT EXISTS idx_approvals_resource
  ON lcc.approvals (resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_approvals_member_status
  ON lcc.approvals (member_id, decision);

-- -----------------------------------------------------------------------------
-- engagement_replies — confirm shape aligns with canonical EngagementTask.
-- 0007 already declares: id, member_id, inbound_message_id (nullable),
-- kind, draft_body, status. Add missing priority_score (for ρ).
-- -----------------------------------------------------------------------------
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc'
                   AND table_name = 'engagement_replies'
                   AND column_name = 'priority_score') THEN
    ALTER TABLE lcc.engagement_replies
      ADD COLUMN priority_score NUMERIC(5, 4);
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc'
                   AND table_name = 'engagement_replies'
                   AND column_name = 'due_at') THEN
    ALTER TABLE lcc.engagement_replies
      ADD COLUMN due_at TIMESTAMPTZ;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc'
                   AND table_name = 'engagement_replies'
                   AND column_name = 'completed_at') THEN
    ALTER TABLE lcc.engagement_replies
      ADD COLUMN completed_at TIMESTAMPTZ;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc'
                   AND table_name = 'engagement_replies'
                   AND column_name = 'target_post_id') THEN
    ALTER TABLE lcc.engagement_replies
      ADD COLUMN target_post_id TEXT;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc'
                   AND table_name = 'engagement_replies'
                   AND column_name = 'version') THEN
    -- F-AUDIT-44 (verified against a real PostgreSQL 15 run): the guard above
    -- tests `engagement_replies` but the ALTER targeted a misspelled
    -- `lcc.engagement_reails`, so the statement failed with
    --     ERROR: relation "lcc.engagement_reails" does not exist
    -- and 0018 aborted before adding the optimistic-concurrency column.
    ALTER TABLE lcc.engagement_replies
      ADD COLUMN version BIGINT NOT NULL DEFAULT 1;
  END IF;
END$$;

CREATE INDEX IF NOT EXISTS idx_engagement_replies_member_status_due
    ON lcc.engagement_replies (member_id, status, due_at NULLS LAST);

-- -----------------------------------------------------------------------------
-- audit log — append-only enforcement at the DB role level
-- (Design §17 / Non-Negotiable §7).
-- The role grant in 0011 already REVOKEs UPDATE/DELETE; this migration adds
-- the rule that even the application role cannot bypass that. Already done
-- in 0011 via REVOKE, but we make it explicit and add a hash-chain column.
-- -----------------------------------------------------------------------------
DO $$
BEGIN
  -- Add `prev_checksum` if missing (0011 declared it but a quick check is cheap).
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                 WHERE table_schema = 'lcc_audit'
                   AND table_name = 'events'
                   AND column_name = 'prev_checksum') THEN
    -- 0011 already declared prev_checksum as TEXT NULL; the DO block is a
    -- safety net for databases where 0011 was applied partially.
    BEGIN
      ALTER TABLE lcc_audit.events ADD COLUMN prev_checksum TEXT;
    EXCEPTION WHEN duplicate_column THEN
      NULL;
    END;
  END IF;
END$$;

-- Append-only enforcement: revoke all privileges except INSERT/SELECT on
-- lcc_audit.events for the application role. Existing 0011 grants SELECT
-- only to lcc_readonly; we tighten the application role here.
DO $$
BEGIN
  -- Defensive: idempotent role-grant audit
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'lcc_app') THEN
    REVOKE UPDATE, DELETE, TRUNCATE ON lcc_audit.events FROM lcc_app;
    REVOKE UPDATE, DELETE, TRUNCATE ON lcc_audit.events_id_seq FROM lcc_app;
    GRANT INSERT, SELECT ON lcc_audit.events TO lcc_app;
  END IF;
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'lcc_audit_writer') THEN
    GRANT INSERT ON lcc_audit.events TO lcc_audit_writer;
    REVOKE UPDATE, DELETE, TRUNCATE ON lcc_audit.events FROM lcc_audit_writer;
  END IF;
END$$;

COMMIT;
