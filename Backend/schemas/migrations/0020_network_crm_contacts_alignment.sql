-- 0020_network_crm_contacts_alignment.sql
-- Bring `lcc.contacts` into line with the canonical contract + Backend Design
-- Concept §11.5, which agree with each other field for field.
--
-- WHY THIS MIGRATION EXISTS
-- -------------------------
-- `lcc.contacts` as created by 0008 is a *pre-contract* shape:
--
--     display_name, headline, company, is_vip, is_mutual,
--     relationship_strength, last_contact_at, metadata
--
-- Three independent artefacts describe the CRM contact, and all three agree
-- with each other and not with 0008:
--
--   1. Backend Design Concept §11.5 `contact`
--   2. Contract/openapi/lcc-api-canonical.yaml  -> components.schemas.Contact
--   3. Backend/proto/lcc/v1/network/network.proto -> message Contact
--
-- 0008 alone disagrees. 0016 and 0017 are the first steps in closing that gap
-- (0016 added `stale`/`stale_since` purely to serve the design's
-- `GET /members/{id}/contacts/stale`; 0017 wired `company_id` to the
-- contract-required `lcc.companies`).  This migration continues that work for
-- the remaining §11.5 fields. It follows the precedent set by 0017's
-- F-AUDIT-43 note, where the stale `lcc.opportunities` table was brought into
-- line with the contract rather than the contract being rewritten to fit it:
--
--     "The mismatch is not the index's fault: both the canonical contract ...
--      and opportunity-svc's own SQL ... expect the contract shape. The 0009
--      table is the stale artifact ... Rather than rewrite the index to match
--      the stale table ... the columns are brought into line with the
--      contract here."
--
-- Every column below is (c) under the wave's semantic standard: required by
-- BOTH the contract and the design docs, and served by no existing column.
-- Nothing here is invented. `email`, `notes`, `last_touched_at`,
-- `last_interaction_kind` and an i16 `connection_strength` are NOT added --
-- see the deliverable for where each of those lives or why it was dropped.
--
-- Deliberately NOT added:
--   * `health_score` (NUMERIC(5,2) in §11.5) - declared in the contract, the
--     design and the proto, but NOTHING in this repository computes or writes
--     it: there is no scoring job, no cron, and no writer. Adding a column
--     that is permanently NULL would be inventing a metric. Flagged as an open
--     question for the product owner instead.
--   * `third_party_ttl_at` - present in §11.5 and the proto but absent from the
--     REST contract, the frontend types, and every writer in the repo. Same
--     reasoning; `lcc.companies.third_party_ttl_at` is the column that is
--     actually used for cached-signal TTL.
--   * `opportunity_id` - §11.5 declares it as a FK, but the schema models the
--     edge in the other direction (`lcc.opportunities.contact_id`, 0009), which
--     is a 1:N relation. A single-valued `contact.opportunity_id` therefore has
--     no unique source; network-crm-svc derives it only when exactly one
--     opportunity is linked and returns NULL otherwise, rather than guessing.
--
-- IDEMPOTENCY: every statement is `IF NOT EXISTS` / `DO $$`-guarded, so this
-- file can be replayed against an already-migrated database.

BEGIN;

-- =============================================================================
-- 1. version - optimistic concurrency
-- =============================================================================
-- Design §52: "Every mutable entity (content_item, sequence, sequence_step,
-- opportunity, **contact**, profile_snapshot) carries an integer `version`
-- column." §11.5 declares `version INTEGER NOT NULL DEFAULT 1`. The contract
-- declares `Contact.version` and `ContactUpdate.expected_version`.
-- `lcc.contacts` had none, so `PATCH /contacts/{id}` could not implement the
-- documented compare-and-swap. This is the same column 0017 added to
-- `lcc.opportunities` (also INT, also DEFAULT 1).
ALTER TABLE lcc.contacts
    ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;

-- =============================================================================
-- 2. tier - VIP / standard / peer
-- =============================================================================
-- Contract `Contact.tier` and `ContactCreate.tier` (default: standard);
-- `ContactUpdate.tier`; proto enum `ContactTier`; §11.5
-- `tier TEXT NOT NULL DEFAULT 'standard' CHECK (tier IN ('VIP','standard','peer'))`.
--
-- `is_vip BOOLEAN` (0008) is NOT a substitute: it can only express 2 of the 3
-- declared states, so mapping `tier` onto it would silently collapse `peer`
-- into `standard` and lose any write of that value. `tier` becomes the
-- authoritative column; `is_vip` is retained as a legacy mirror because other
-- consumers (and 0008's own defaults) still read it. The backfill below seeds
-- it, and network-crm-svc keeps the two in lockstep on every write.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc' AND table_name = 'contacts' AND column_name = 'tier'
  ) THEN
    ALTER TABLE lcc.contacts
        ADD COLUMN tier TEXT NOT NULL DEFAULT 'standard'
        CONSTRAINT contacts_tier_check CHECK (tier IN ('VIP', 'standard', 'peer'));
  END IF;
END$$;

UPDATE lcc.contacts
   SET tier = CASE WHEN is_vip THEN 'VIP' ELSE 'standard' END
 WHERE tier IS DISTINCT FROM CASE WHEN is_vip THEN 'VIP' ELSE 'standard' END;

-- =============================================================================
-- 3. connection_status - not_connected / pending / connected
-- =============================================================================
-- Contract `Contact.connection_status`; proto enum `ConnectionStatus`; §11.5
-- `connection_status TEXT NOT NULL CHECK (connection_status IN
-- ('not_connected','pending','connected'))`.
--
-- `is_mutual BOOLEAN` is a *relationship* fact (do we both know each other),
-- not a *connection request* state, and it cannot express `pending`. The two
-- are left independent rather than derived from one another.
--
-- Seeded from `is_mutual` because that is the closest truth the existing data
-- carries: a mutual connection is, by definition, `connected`.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc' AND table_name = 'contacts'
      AND column_name = 'connection_status'
  ) THEN
    ALTER TABLE lcc.contacts
        ADD COLUMN connection_status TEXT NOT NULL DEFAULT 'not_connected'
        CONSTRAINT contacts_connection_status_check
        CHECK (connection_status IN ('not_connected', 'pending', 'connected'));
  END IF;
END$$;

UPDATE lcc.contacts
   SET connection_status = 'connected'
 WHERE is_mutual AND connection_status = 'not_connected';

-- =============================================================================
-- 4. linkedin_url
-- =============================================================================
-- Contract `Contact.linkedin_url` (`format: uri`); proto field 3; §11.5
-- `linkedin_url TEXT` with `UNIQUE (member_id, linkedin_url)`.
--
-- NOT the same thing as 0008's `linkedin_id TEXT`. `linkedin_id` is the
-- platform's member id (a short opaque key, see `idx_contacts_linkedin`);
-- `linkedin_url` is the human-facing profile URI. Both are kept: the browser
-- extension and the events pipeline key off `linkedin_id`, the CRM UI links to
-- `linkedin_url`.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc' AND table_name = 'contacts'
      AND column_name = 'linkedin_url'
  ) THEN
    ALTER TABLE lcc.contacts ADD COLUMN linkedin_url TEXT;
  END IF;
END$$;

-- Partial unique index rather than a table UNIQUE constraint: NULLs must stay
-- comparable (a member may hold many contacts imported without a profile URL),
-- and a plain UNIQUE would still pass but the partial form also keeps the
-- index small. Mirrors the partial-index style used throughout 0017.
CREATE UNIQUE INDEX IF NOT EXISTS uq_contacts_member_linkedin_url
    ON lcc.contacts (member_id, linkedin_url)
    WHERE linkedin_url IS NOT NULL;

-- =============================================================================
-- 5. relationship_stage - cold .. closed
-- =============================================================================
-- Contract `Contact.relationship_stage` and `ContactUpdate.relationship_stage`;
-- proto enum `RelationshipStage`; §11.5
-- `relationship_stage TEXT NOT NULL DEFAULT 'cold' CHECK (relationship_stage
-- IN ('cold','connected','engaged','conversation','opportunity','closed'))`.
--
-- Deliberately NOT derived from `relationship_strength`. The strength enum
-- (none/weak/medium/strong/strong_recent) measures closeness; the stage is a
-- pipeline position the user drives. Conflating them would report a stage the
-- user never set. The user writes it; it starts at `cold`.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc' AND table_name = 'contacts'
      AND column_name = 'relationship_stage'
  ) THEN
    ALTER TABLE lcc.contacts
        ADD COLUMN relationship_stage TEXT NOT NULL DEFAULT 'cold'
        CONSTRAINT contacts_relationship_stage_check
        CHECK (relationship_stage IN
               ('cold', 'connected', 'engaged', 'conversation', 'opportunity', 'closed'));
  END IF;
END$$;

-- =============================================================================
-- 6. first_contact_date / follow_up_date
-- =============================================================================
-- Contract `ContactCreate.first_contact_date`, `Contact.first_contact_date`,
-- `ContactUpdate.follow_up_date`, `Contact.follow_up_date`; §11.5
-- `first_contact_date DATE` and `follow_up_date DATE`.
--
-- `last_contact_at` (0008) is deliberately NOT reused for `first_contact_date`:
-- it is a rolling "last touched" stamp that `record_interaction` overwrites,
-- so a MIN()/MAX() substitution would change meaning on every write.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc' AND table_name = 'contacts'
      AND column_name = 'first_contact_date'
  ) THEN
    ALTER TABLE lcc.contacts ADD COLUMN first_contact_date DATE;
  END IF;
END$$;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc' AND table_name = 'contacts'
      AND column_name = 'follow_up_date'
  ) THEN
    ALTER TABLE lcc.contacts ADD COLUMN follow_up_date DATE;
  END IF;
END$$;

-- §11.5 indexes: member+stage is the list default, follow_up_date is the
-- reminder sweep. Both partial/ordered to match the surrounding migrations.
CREATE INDEX IF NOT EXISTS idx_contacts_member_stage
    ON lcc.contacts (member_id, relationship_stage);
CREATE INDEX IF NOT EXISTS idx_contacts_follow_up
    ON lcc.contacts (follow_up_date) WHERE follow_up_date IS NOT NULL;

COMMIT;
