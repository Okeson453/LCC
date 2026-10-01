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
CREATE MATERIALIZED VIEW lcc.analytics_member_daily AS
SELECT
  m.id AS member_id,
  DATE_TRUNC('day', NOW()) AS day,
  COALESCE(SUM((cm.metadata->>'action_count')::INT), 0) AS total_actions
FROM lcc.members m
LEFT JOIN lcc.content_items ci ON ci.member_id = m.id
LEFT JOIN LATERAL JSONB_ARRAY_ELEMENTS(ci.metadata->'metrics') AS cm ON TRUE
GROUP BY m.id;
CREATE UNIQUE INDEX idx_analytics_member_daily ON lcc.analytics_member_daily(member_id, day);

COMMIT;
