-- 0010_compliance.sql
-- Compliance configuration, restrictions, daily caps, cooldowns.

BEGIN;

CREATE TABLE lcc.compliance_config_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    version TEXT NOT NULL UNIQUE,             -- 'ccfg-2025-01-15-rc1'
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    config JSONB NOT NULL,                    -- full validated compliance config JSON
    activated_at TIMESTAMPTZ,
    activated_by TEXT,
    two_reviewer_signed_by TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX idx_one_active_config ON lcc.compliance_config_versions((is_active)) WHERE is_active = TRUE;

CREATE TABLE lcc.account_health_snapshots (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    h_c_score NUMERIC(4, 3) NOT NULL,
    band TEXT NOT NULL,
    components JSONB NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_account_health_member ON lcc.account_health_snapshots(member_id, observed_at DESC);
SELECT lcc.attach_member_rls('account_health_snapshots');

CREATE TABLE lcc.daily_caps (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    date DATE NOT NULL,
    action_type TEXT NOT NULL,
    used INT NOT NULL DEFAULT 0,
    cap INT NOT NULL,
    ab_d_multiplier NUMERIC(4, 3) NOT NULL DEFAULT 1.0,
    UNIQUE (member_id, date, action_type)
);
SELECT lcc.attach_member_rls('daily_caps');

CREATE TABLE lcc.cooldown_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    action_type TEXT NOT NULL,
    target_kind TEXT NOT NULL,
    target_id TEXT NOT NULL,
    cooldown_until TIMESTAMPTZ NOT NULL,
    set_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (member_id, action_type, target_kind, target_id)
);
CREATE INDEX idx_cooldown_until ON lcc.cooldown_rules(member_id, cooldown_until);
SELECT lcc.attach_member_rls('cooldown_rules');

CREATE TABLE lcc.restrictions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    signal_kind TEXT NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cleared_at TIMESTAMPTZ,
    raw_response_snippet TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB
);
CREATE INDEX idx_restrictions_member ON lcc.restrictions(member_id, detected_at DESC);
SELECT lcc.attach_member_rls('restrictions');

COMMIT;
