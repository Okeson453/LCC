-- 0016_contact_staleness.sql
-- Audit F-AUDIT-13: the staleness scanner computed staleness entirely in a
-- SELECT COUNT(*) and discarded the result, so `GET /members/{id}/contacts/stale`
-- (Backend Design Concept §18.5) had no data source. The scanner now persists
-- its verdict on the contact row.
--
-- `stale` / `stale_since` are denormalised onto lcc.contacts so the endpoint is
-- a single indexed read rather than a full scan with a per-tier CASE
-- expression on every request.

BEGIN;

ALTER TABLE lcc.contacts
    ADD COLUMN IF NOT EXISTS stale BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS stale_since TIMESTAMPTZ;

-- Partial index: the endpoint only ever asks for `stale = TRUE`, which is a
-- small fraction of the contact graph.
CREATE INDEX IF NOT EXISTS idx_contacts_stale
    ON lcc.contacts (member_id, last_contact_at)
    WHERE stale;

COMMIT;
