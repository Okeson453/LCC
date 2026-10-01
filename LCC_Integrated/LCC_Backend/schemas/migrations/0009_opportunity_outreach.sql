-- 0009_opportunity_outreach.sql
-- Opportunities, signals, outreach drafts.

BEGIN;

CREATE TYPE lcc.opportunity_kind AS ENUM (
    'job_posting', 'consulting', 'partnership', 'speaking', 'mentorship', 'other'
);

CREATE TYPE lcc.opportunity_funnel AS ENUM (
    'discovered', 'qualified', 'in_conversation', 'proposed', 'negotiating', 'won', 'lost'
);

CREATE TABLE lcc.opportunities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    kind lcc.opportunity_kind NOT NULL,
    funnel lcc.opportunity_funnel NOT NULL DEFAULT 'discovered',
    title TEXT NOT NULL,
    company TEXT,
    contact_id UUID REFERENCES lcc.contacts(id) ON DELETE SET NULL,
    phi_score NUMERIC(4, 3) NOT NULL DEFAULT 0,
    phi_components JSONB NOT NULL DEFAULT '{}'::JSONB,
    last_signal_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_opportunities_member ON lcc.opportunities(member_id);
CREATE INDEX idx_opportunities_funnel ON lcc.opportunities(funnel);
SELECT lcc.attach_member_rls('opportunities');

CREATE TYPE lcc.signal_kind AS ENUM (
    'job_change', 'funding_event', 'post_about_problem',
    'competitor_mention', 'manual', 'company_growth', 'hiring'
);

CREATE TABLE lcc.opportunity_signals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    opportunity_id UUID REFERENCES lcc.opportunities(id) ON DELETE CASCADE,
    kind lcc.signal_kind NOT NULL,
    weight NUMERIC(4, 3) NOT NULL DEFAULT 0,
    text TEXT NOT NULL DEFAULT '',
    matched_kb_facts INT NOT NULL DEFAULT 0,
    observed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ttl_expires_at TIMESTAMPTZ,                  -- NULL = first-party
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB
);
CREATE INDEX idx_opp_signals_opp ON lcc.opportunity_signals(opportunity_id, observed_at DESC);
SELECT lcc.attach_member_rls('opportunity_signals');

CREATE TABLE lcc.outreach_drafts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES lcc.contacts(id) ON DELETE SET NULL,
    opportunity_id UUID REFERENCES lcc.opportunities(id) ON DELETE SET NULL,
    body TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'drafted',
    sent_at TIMESTAMPTZ,
    linkedin_event_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
SELECT lcc.attach_member_rls('outreach_drafts');

COMMIT;
