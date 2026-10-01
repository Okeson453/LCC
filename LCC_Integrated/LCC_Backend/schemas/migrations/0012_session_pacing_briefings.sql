-- 0012_session_pacing_briefings.sql
-- Session pacing (per-tenant daily rolling budgets), briefings.

BEGIN;

CREATE TABLE lcc.session_pacing (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    window_start TIMESTAMPTZ NOT NULL,
    window_seconds INT NOT NULL,
    actions_in_window INT NOT NULL DEFAULT 0,
    cap_in_window INT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_session_pacing_member ON lcc.session_pacing(member_id, window_start DESC);
SELECT lcc.attach_member_rls('session_pacing');

CREATE TYPE lcc.briefing_kind AS ENUM ('morning', 'midday', 'evening', 'custom');

CREATE TABLE lcc.briefings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    kind lcc.briefing_kind NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delivered_at TIMESTAMPTZ,
    body JSONB NOT NULL,
    payload_ref TEXT,
    status TEXT NOT NULL DEFAULT 'generated'
);
CREATE INDEX idx_briefings_member ON lcc.briefings(member_id, generated_at DESC);
SELECT lcc.attach_member_rls('briefings');

COMMIT;
