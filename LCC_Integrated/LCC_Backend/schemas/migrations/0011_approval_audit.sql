-- 0011_approval_audit.sql
-- Approvals, audit log (insert-only).

BEGIN;

CREATE TYPE lcc.approval_decision AS ENUM ('pending', 'approved', 'rejected', 'expired', 'withdrawn');

CREATE TABLE lcc.approvals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    resource_type TEXT NOT NULL,             -- 'content_item' | 'outreach_draft' | 'sequence_step' | 'profile_edit'
    resource_id UUID NOT NULL,
    requested_action JSONB NOT NULL,
    decision lcc.approval_decision NOT NULL DEFAULT 'pending',
    decided_by TEXT,
    decided_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    decision_rationale TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_approvals_member ON lcc.approvals(member_id);
CREATE INDEX idx_approvals_pending ON lcc.approvals(expires_at) WHERE decision = 'pending';
SELECT lcc.attach_member_rls('approvals');

-- ---- Audit log: INSERT-only by append-only role ----
-- Per Non-Negotiable §10 and the audit_db_role constraint,
-- audit tables only allow INSERT; UPDATE/DELETE are revoked.

CREATE TABLE lcc_audit.events (
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL UNIQUE,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    member_id UUID,
    outcome TEXT NOT NULL,
    reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    checksum_sha256 TEXT NOT NULL,
    prev_checksum TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    trace_id TEXT
);
CREATE INDEX idx_audit_actor ON lcc_audit.events(actor, occurred_at DESC);
CREATE INDEX idx_audit_member ON lcc_audit.events(member_id, occurred_at DESC);
CREATE INDEX idx_audit_resource ON lcc_audit.events(resource_type, resource_id);

-- INSERT-only for lcc_audit_writer; service roles use lcc_app which is denied DELETE/UPDATE.
REVOKE ALL ON lcc_audit.events FROM PUBLIC;
GRANT INSERT ON lcc_audit.events TO lcc_audit_writer;
GRANT INSERT ON lcc_audit.events TO lcc_app;
GRANT USAGE, SELECT ON SEQUENCE lcc_audit.events_id_seq TO lcc_app;

-- App role cannot read audit logs directly; only the audit-svc reads them.
-- (Audit-svc connects as the lcc_readonly role.)

-- Idempotency store (per service) lives in the lcc schema.
CREATE TABLE lcc.idempotency_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE (member_id, key, resource_type)
);
CREATE INDEX idx_idempotency_expiry ON lcc.idempotency_keys(expires_at);
SELECT lcc.attach_member_rls('idempotency_keys');

COMMIT;
