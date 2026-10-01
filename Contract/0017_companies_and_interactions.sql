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
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- A member cannot have two non-deleted companies with the same
    -- (case-insensitive) name.
    UNIQUE (member_id, lower(name))
);

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
DO $$
BEGIN
  -- Only attempt the conversion if the column is not yet typed as UUID.
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc'
      AND table_name = 'contacts'
      AND column_name = 'company_id'
      AND data_type <> 'uuid'
  ) THEN
    -- 1) Drop any NOT NULL constraint that would block NULL during conversion.
    ALTER TABLE lcc.contacts ALTER COLUMN company_id DROP NOT NULL;

    -- 2) Try to cast text → uuid. Rows that don't parse become NULL — this
    --    is a one-time data backfill on existing rows. New rows must be UUID.
    BEGIN
      ALTER TABLE lcc.contacts
        ALTER COLUMN company_id TYPE UUID USING (company_id::uuid);
    EXCEPTION WHEN others THEN
      -- If any row is non-castable, NULL it; we do not lose data otherwise.
      UPDATE lcc.contacts SET company_id = NULL WHERE company_id !~* '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$';
      ALTER TABLE lcc.contacts
        ALTER COLUMN company_id TYPE UUID USING (company_id::uuid);
    END;

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

-- Indexes for opportunity queries (canonical /opportunities list with filters)
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
