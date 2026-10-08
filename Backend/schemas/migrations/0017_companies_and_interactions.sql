-- 0017_companies_and_interactions.sql
-- Phase B — DB foundation gap closure per contract_audit.
--
-- Two tables required by the canonical contract are referenced by other
-- tables but not defined anywhere in the repository:
--
--   1. lcc.companies  — referenced by lcc.contacts.company_id (FOREIGN KEY
--      present in design §11.6; missing in migrations). The design also
--      declares opportunities.company_id and message_templates under
--      company context. This migration creates the table with the columns
--      the canonical contract (Contract Endpoint Matrix §G7) requires and
--      retroactively re-points lcc.contacts.company_id to it.
--
--   2. lcc.interactions — required by the canonical contract's
--      `/contacts/{id}/interactions` endpoint family. Without this table
--      the list and create handlers have no destination. The existing
--      `lcc.inbound_messages` is for inbound traffic only; `interactions`
--      is the symmetric log of every contact-touching event (manual note,
--      call, meeting, sent message, etc.) per design §3.4.
--
-- Both tables follow the existing conventions:
--   • `id UUID PRIMARY KEY DEFAULT uuid_generate_v4()`
--   • `member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE`
--   • `created_at/updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
--   • `version BIGINT NOT NULL DEFAULT 1` (optimistic concurrency)
--   • RLS via `lcc.attach_member_rls(...)` — already fixed in 0002

BEGIN;

-- =============================================================================
-- 1. lcc.companies
-- =============================================================================
CREATE TABLE IF NOT EXISTS lcc.companies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    domain TEXT,
    industry TEXT,
    size_band TEXT,                              -- '1-10' | '11-50' | '51-200' | '201-500' | '501-1000' | '1000+'
    funding_stage TEXT,                          -- 'pre_seed' | 'seed' | 'series_a' | ... | 'public' | NULL
    hq_location TEXT,
    tech_stack TEXT[] NOT NULL DEFAULT '{}',
    trigger_events JSONB NOT NULL DEFAULT '[]'::JSONB,
    public_signals JSONB NOT NULL DEFAULT '[]'::JSONB,
    enrichment_meta JSONB NOT NULL DEFAULT '{}'::JSONB,
    third_party_ttl_at TIMESTAMPTZ,              -- ≤48h for cached signals
    deleted_at TIMESTAMPTZ,                      -- soft delete
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- F-AUDIT-37 (verified against a real PostgreSQL 15 run): the case-insensitive
-- uniqueness rule was written as a table-level `UNIQUE (member_id, lower(name))`
-- constraint. A table constraint may only contain plain column references, not
-- expressions, so the statement failed to parse:
--     ERROR: syntax error at or near "("
-- and 0017 — and therefore the two contract-required tables `lcc.companies` and
-- `lcc.interactions` — was never created.
--
-- The correct construct is a unique INDEX over the expression, which does
-- enforce the intended rule. The partial predicate additionally implements the
-- design's "non-deleted" requirement: soft-deleting a company frees the name.
CREATE UNIQUE INDEX IF NOT EXISTS uq_companies_member_name_live
    ON lcc.companies (member_id, lower(name))
    WHERE deleted_at IS NULL;

-- Most reads filter by member; partial unique prevents the same name from
-- being inserted twice for the same member.
CREATE INDEX IF NOT EXISTS idx_companies_member ON lcc.companies (member_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_companies_industry ON lcc.companies (member_id, industry) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_companies_size ON lcc.companies (member_id, size_band) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_companies_funding ON lcc.companies (member_id, funding_stage) WHERE deleted_at IS NULL;

-- RLS — the standard isolation pattern.
SELECT lcc.attach_member_rls('companies');

-- updated_at trigger (uses the helper from 0014 if present, else creates it)
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_proc WHERE proname = 'touch_updated_at' AND pronamespace = 'lcc'::regnamespace) THEN
    CREATE OR REPLACE FUNCTION lcc.touch_updated_at() RETURNS TRIGGER AS $inner$
    BEGIN
      NEW.updated_at = NOW();
      RETURN NEW;
    END;
    $inner$ LANGUAGE plpgsql;
  END IF;
END$$;

DROP TRIGGER IF EXISTS companies_touch ON lcc.companies;
CREATE TRIGGER companies_touch BEFORE UPDATE ON lcc.companies
    FOR EACH ROW EXECUTE FUNCTION lcc.touch_updated_at();

-- =============================================================================
-- 2. Wire lcc.contacts.company_id → lcc.companies.id
--    (current state: lcc.contacts.company_id is typed TEXT and not FK-bound —
--     per a quick read of 0008, no FK is declared, so the column is just text.
--     The canonical contract requires it to be a UUID reference.)
-- =============================================================================
-- 2. Link contacts → companies
-- -----------------------------------------------------------------------------
-- F-AUDIT-41 (verified against a real PostgreSQL 15 run): the whole block below
-- was guarded by
--     IF EXISTS (... column_name = 'company_id' AND data_type <> 'uuid')
-- i.e. "convert company_id from text to uuid *if it is already there*". But
-- `lcc.contacts` (0008) never had a `company_id` column at all — only a
-- free-text `company` — so the condition was false, the entire conversion and
-- FK block was skipped, and the very next unguarded statement failed:
--     ERROR: column "company_id" does not exist
--     HINT: Perhaps you meant to reference the column "contacts.company"
-- Because 0017 aborted, neither `lcc.companies` nor `lcc.interactions` was
-- ever created, and `PATCH /api/v1/contacts/{id}/company` — declared in the
-- canonical contract and implemented in network-crm-svc against
-- `lcc.contacts.company_id` — had no column to write to.
--
-- The condition was written for the wrong starting state. `company_id` must be
-- ADDED (it is the contract-required relation to `lcc.companies.id`; the
-- existing `company` TEXT column is a denormalised display name and is kept),
-- and the FK added once the column is UUID. This is now idempotent and
-- handles all three states: absent, present-as-text, present-as-uuid.
DO $$
BEGIN
  -- 1) Add the column when missing. Contract ContactCreate.company_id is a
  --    uuid; network-crm-svc binds it as Option<Uuid>.
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc' AND table_name = 'contacts' AND column_name = 'company_id'
  ) THEN
    ALTER TABLE lcc.contacts ADD COLUMN company_id UUID;
  END IF;

  -- 2) If it exists as text (legacy deployments), convert it to uuid.
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc' AND table_name = 'contacts'
      AND column_name = 'company_id' AND data_type <> 'uuid'
  ) THEN
    ALTER TABLE lcc.contacts ALTER COLUMN company_id DROP NOT NULL;
    BEGIN
      ALTER TABLE lcc.contacts
        ALTER COLUMN company_id TYPE UUID USING (company_id::uuid);
    EXCEPTION WHEN others THEN
      -- Non-castable legacy values are dropped rather than failing the release.
      UPDATE lcc.contacts SET company_id = NULL
        WHERE company_id IS NOT NULL
          AND company_id !~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$';
      ALTER TABLE lcc.contacts
        ALTER COLUMN company_id TYPE UUID USING (company_id::uuid);
    END;
  END IF;

  -- 3) Add the FK now that the column is UUID.
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.table_constraints
    WHERE table_schema = 'lcc'
      AND table_name = 'contacts'
      AND constraint_type = 'FOREIGN KEY'
      AND constraint_name = 'contacts_company_id_fkey'
  ) THEN
    ALTER TABLE lcc.contacts
      ADD CONSTRAINT contacts_company_id_fkey
      FOREIGN KEY (company_id) REFERENCES lcc.companies(id) ON DELETE SET NULL;
  END IF;
END$$;

CREATE INDEX IF NOT EXISTS idx_contacts_company ON lcc.contacts (company_id) WHERE company_id IS NOT NULL;

-- =============================================================================
-- 3. lcc.interactions
-- =============================================================================
CREATE TABLE IF NOT EXISTS lcc.interactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES lcc.contacts(id) ON DELETE CASCADE,
    -- 'manual_note' for human-entered journal entries; the rest are
    -- auto-logged by services when a corresponding event is emitted.
    kind TEXT NOT NULL CHECK (kind IN (
        'manual_note', 'inbound_message', 'sent_message',
        'call', 'meeting', 'sequence_step_sent',
        'reply_received', 'connection_accepted'
    )),
    summary TEXT NOT NULL,
    -- Optional references to domain entities that triggered the interaction
    -- (sequence_step_id, message_id, etc.) — nullable so manual notes fit.
    related_sequence_id UUID,
    related_sequence_step_id UUID,
    related_content_item_id UUID,
    related_approval_id UUID,
    -- Author of the interaction. 'member' for human-entered; 'system:<svc>'
    -- for auto-logged events.
    actor TEXT NOT NULL DEFAULT 'member',
    occurred_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- The endpoint `/contacts/{id}/interactions` is the primary reader; it
-- queries by contact_id with ORDER BY occurred_at DESC.
CREATE INDEX IF NOT EXISTS idx_interactions_contact_occurred
    ON lcc.interactions (contact_id, occurred_at DESC)
    WHERE deleted_at IS NULL;

-- The member-scoped queries (audit, exports) use this.
CREATE INDEX IF NOT EXISTS idx_interactions_member_occurred
    ON lcc.interactions (member_id, occurred_at DESC)
    WHERE deleted_at IS NULL;

SELECT lcc.attach_member_rls('interactions');

-- =============================================================================
-- 4. Wire opportunity_signals / applications / sequences that reference
--    contacts/companies via text columns
-- =============================================================================
-- A handful of tables in 0009 (opportunities, outreach_drafts) use TEXT for
-- contact_id rather than UUID. The canonical contract requires UUID. Apply
-- the same defensive conversion as above.
DO $$
DECLARE
    rec record;
BEGIN
  FOR rec IN
    SELECT table_schema, table_name, column_name
    FROM information_schema.columns
    WHERE table_schema = 'lcc'
      AND column_name IN ('contact_id', 'company_id')
      AND data_type <> 'uuid'
  LOOP
    EXECUTE format('ALTER TABLE %I.%I ALTER COLUMN %I DROP NOT NULL',
                   rec.table_schema, rec.table_name, rec.column_name);
    BEGIN
      EXECUTE format('ALTER TABLE %I.%I ALTER COLUMN %I TYPE UUID USING (%I::uuid)',
                     rec.table_schema, rec.table_name, rec.column_name, rec.column_name);
    EXCEPTION WHEN others THEN
      EXECUTE format('UPDATE %I.%I SET %I = NULL WHERE %I !~* ''^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$''',
                     rec.table_schema, rec.table_name, rec.column_name, rec.column_name);
      EXECUTE format('ALTER TABLE %I.%I ALTER COLUMN %I TYPE UUID USING (%I::uuid)',
                     rec.table_schema, rec.table_name, rec.column_name, rec.column_name);
    END;
  END LOOP;
END$$;

-- F-AUDIT-43 (verified against a real PostgreSQL 15 run of the full migration
-- set): this index referenced `opportunities.status` and `opportunities.fit_score`,
-- but `lcc.opportunities` as created by 0009 carries neither. Its real columns
-- are: id, member_id, kind, funnel, title, company, contact_id, phi_score,
-- phi_components, last_signal_at, metadata, created_at, updated_at.
--
-- So 0017 failed with
--     ERROR: column "status" does not exist
-- and aborted — which is why `lcc.companies` and `lcc.interactions`, the two
-- tables the canonical contract requires, were never created.
--
-- The mismatch is not the index's fault: both the canonical contract
-- (`components.schemas.Opportunity`: `type`, `source`, `status`, `fit_score`,
-- `company_id`, `discovered_at`, `version`) and opportunity-svc's own SQL
-- (`SELECT ... company_id, source::TEXT, status::TEXT, fit_score,
-- discovered_at, ... version`) expect the contract shape. The 0009 table is the
-- stale artifact — it predates the contract and was never brought forward.
--
-- Rather than rewrite the index to match the stale table (which would leave
-- every opportunity-svc query broken at runtime), the columns are brought into
-- line with the contract here. `funnel`/`phi_score` are retained as
-- compatibility aliases so no existing consumer loses data, and the
-- service-facing names become the source of truth.
ALTER TABLE lcc.opportunities
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'discovered',
    ADD COLUMN IF NOT EXISTS fit_score DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS company_id UUID REFERENCES lcc.companies(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'manual',
    ADD COLUMN IF NOT EXISTS discovered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS last_evaluated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS version INT NOT NULL DEFAULT 1;

-- Backfill the renamed columns from their 0009 equivalents so pre-existing rows
-- keep their meaning: `funnel` carried the stage (an enum type), `phi_score`
-- the fit score. `funnel` is cast to text because `opportunity_funnel` and
-- `status` are different types and COALESCE requires a common one.
UPDATE lcc.opportunities
   SET fit_score = COALESCE(fit_score, phi_score),
       status   = COALESCE(NULLIF(status, 'discovered'), funnel::TEXT)
 WHERE fit_score IS NULL OR funnel IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_opportunities_member_status_fit
    ON lcc.opportunities (member_id, status, fit_score DESC NULLS LAST)
    WHERE status IN ('discovered', 'qualified', 'contacted', 'conversation');

-- Idempotency: per (member, application, application_type) uniqueness is
-- declared in 0014. Add a partial unique index for the queued/active rows.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema='lcc' AND table_name='applications') THEN
    EXECUTE 'CREATE UNIQUE INDEX IF NOT EXISTS uniq_applications_member_opp_type_active
             ON lcc.applications (member_id, opportunity_id, application_type)
             WHERE deleted_at IS NULL';
  END IF;
END$$;

COMMIT;
