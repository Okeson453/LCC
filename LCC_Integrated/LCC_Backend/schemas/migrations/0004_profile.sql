-- 0004_profile.sql
-- Profile, profile snapshots, profile audits.

BEGIN;

CREATE TABLE lcc.profile_snapshots (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    source TEXT NOT NULL,  -- 'linkedin_api' | 'browser_extension' | 'manual'
    snapshot_kind TEXT NOT NULL,  -- 'profile' | 'about' | 'experience' | 'skills' | 'recommendations'
    data JSONB NOT NULL,
    third_party_ttl_expires_at TIMESTAMPTZ,  -- NULL when first-party
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_current BOOLEAN NOT NULL DEFAULT TRUE
);
CREATE INDEX idx_profile_snapshots_member ON lcc.profile_snapshots(member_id);
SELECT lcc.attach_member_rls('profile_snapshots');

CREATE TABLE lcc.profile_audits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    snapshot_id UUID REFERENCES lcc.profile_snapshots(id) ON DELETE SET NULL,
    audit_kind TEXT NOT NULL,  -- 'gap_analysis' | 'messaging_review' | 'visibility'
    score NUMERIC(4, 3) NOT NULL,
    findings JSONB NOT NULL,
    recommendations JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
SELECT lcc.attach_member_rls('profile_audits');

COMMIT;
