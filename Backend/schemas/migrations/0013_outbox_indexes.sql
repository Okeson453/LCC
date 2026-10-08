-- 0013_outbox_indexes.sql
-- Outbox pattern indexes + materialized views for analytics.

BEGIN;

-- Outbox for at-least-once event publishing.
CREATE TABLE lcc.outbox (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    topic TEXT NOT NULL,
    payload JSONB NOT NULL,
    trace_id TEXT,
    idempotency_key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ
);
CREATE INDEX idx_outbox_pending ON lcc.outbox(created_at) WHERE published_at IS NULL;

-- Materialized view for analytics aggregation.
--
-- F-AUDIT-39 (verified against a real PostgreSQL 15 run): the view referenced
-- a `cm` column that the LATERAL join never projects. `JSONB_ARRAY_ELEMENTS(...)`
-- yields a set of scalars named after the *column* (`value` by default), not
-- `cm`, so the statement failed with
--     ERROR: column cm.metadata does not exist
-- and 0013 aborted — taking the outbox table (and the RLS/seed cascade after
-- it) with it.
--
-- Fixed by extracting each element as a named `metrics` column and reading
-- `action_count` from that element, preserving the original semantics: sum
-- the per-metric `action_count` of every content item's `metadata->'metrics'`
-- array.
--
-- Guarded so the view is only created when content_items exists and carries a
-- metadata column, keeping this migration independent of creation order.
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_schema = 'lcc' AND table_name = 'content_items' AND column_name = 'metadata'
  ) THEN
    EXECUTE $mv$
      CREATE MATERIALIZED VIEW IF NOT EXISTS lcc.analytics_member_daily AS
      SELECT
        m.id AS member_id,
        DATE_TRUNC('day', NOW()) AS day,
        COALESCE(SUM((metrics.value->>'action_count')::INT), 0) AS total_actions
      FROM lcc.members m
      LEFT JOIN lcc.content_items ci ON ci.member_id = m.id
      LEFT JOIN LATERAL jsonb_array_elements(COALESCE(ci.metadata->'metrics', '[]'::JSONB))
           AS metrics(value) ON TRUE
      GROUP BY m.id
    $mv$;
    EXECUTE 'CREATE UNIQUE INDEX IF NOT EXISTS idx_analytics_member_daily ON lcc.analytics_member_daily(member_id, day)';
  END IF;
END$$;

COMMIT;
