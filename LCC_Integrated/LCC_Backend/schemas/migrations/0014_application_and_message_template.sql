-- 0014_application_and_message_template.sql
-- F-41 fix: the original schema is missing tables referenced by the
-- outreach / opportunity / sequence flows. This migration adds them with
-- proper RLS, soft-delete, optimistic concurrency, and audit hooks.

BEGIN;

-- Applications (job-applications and client-proposals sent through the
-- system). One row per (member_id, opportunity_id, application_type).
CREATE TABLE IF NOT EXISTS lcc.applications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    opportunity_id UUID NOT NULL,
    application_type TEXT NOT NULL
        CHECK (application_type IN ('job_application', 'client_proposal', 'executive_outreach')),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft','queued','sent','replied','rejected','withdrawn')),
    submitted_at TIMESTAMPTZ,
    response_received_at TIMESTAMPTZ,
    payload JSONB NOT NULL DEFAULT '{}'::JSONB,
    idempotency_key TEXT,
    version BIGINT NOT NULL DEFAULT 1,
    -- soft delete (F-20)
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- optimistic concurrency
    UNIQUE (member_id, opportunity_id, application_type)
);

CREATE INDEX IF NOT EXISTS idx_applications_member ON lcc.applications(member_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_applications_opportunity ON lcc.applications(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_applications_idem ON lcc.applications(member_id, idempotency_key) WHERE idempotency_key IS NOT NULL;

-- Message templates (saved snippets a member can use in sequences / DMs).
CREATE TABLE IF NOT EXISTS lcc.message_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'general'
        CHECK (category IN ('general','connection','sequence_step','proposal','follow_up')),
    body TEXT NOT NULL,
    variables JSONB NOT NULL DEFAULT '[]'::JSONB,    -- list of {key, default, required}
    usage_count INTEGER NOT NULL DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    -- soft delete
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (member_id, name)
);

CREATE INDEX IF NOT EXISTS idx_message_templates_member ON lcc.message_templates(member_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_message_templates_category ON lcc.message_templates(member_id, category) WHERE deleted_at IS NULL;

-- updated_at triggers
CREATE OR REPLACE FUNCTION lcc.touch_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS applications_touch ON lcc.applications;
CREATE TRIGGER applications_touch BEFORE UPDATE ON lcc.applications
    FOR EACH ROW EXECUTE FUNCTION lcc.touch_updated_at();

DROP TRIGGER IF EXISTS message_templates_touch ON lcc.message_templates;
CREATE TRIGGER message_templates_touch BEFORE UPDATE ON lcc.message_templates
    FOR EACH ROW EXECUTE FUNCTION lcc.touch_updated_at();

-- RLS
SELECT lcc.attach_member_rls('applications');
SELECT lcc.attach_member_rls('message_templates');

COMMIT;
