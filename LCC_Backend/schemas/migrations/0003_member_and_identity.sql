-- 0003_member_and_identity.sql
-- Member, OAuth token, JWT signing key registry.

BEGIN;

CREATE TABLE lcc.members (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    linkedin_id TEXT UNIQUE NOT NULL,
    email CITEXT UNIQUE,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member',  -- 'member' | 'admin' | 'compliance_reviewer' | 'super_admin'
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    locale TEXT NOT NULL DEFAULT 'en-US',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deactivated_at TIMESTAMPTZ
);
SELECT lcc.attach_member_rls('members');
-- Members table is keyed by id, not by member_id — RLS policy applied is "always allow the row matching your id".

CREATE TABLE lcc.oauth_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,  -- 'linkedin' | 'google' | 'github'
    access_token_encrypted BYTEA NOT NULL,
    refresh_token_encrypted BYTEA,
    expires_at TIMESTAMPTZ NOT NULL,
    scope TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (member_id, provider)
);
CREATE INDEX idx_oauth_tokens_member ON lcc.oauth_tokens(member_id);
SELECT lcc.attach_member_rls('oauth_tokens');

CREATE TABLE lcc.consents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    granted_scopes TEXT[] NOT NULL,
    document_version TEXT NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);
SELECT lcc.attach_member_rls('consents');

-- Member-scoped RLS on members uses the `id` pattern: a member can only see
-- their own row. This is enforced by setting `app.current_member_id` to the
-- member's UUID::text before each query.
--
-- F-AUDIT-35: the previous statement was
--     ALTER POLICY member_isolation_members ON lcc.members
--       USING (id::text = current_setting('app.current_member_id', true));
-- `ALTER POLICY ... USING` replaces ONLY the USING expression. The WITH CHECK
-- created by `attach_member_rls` in 0002 — `WITH CHECK (member_id::text =
-- current_setting(...))` — survived, and `lcc.members` has no `member_id`
-- column, so every INSERT or UPDATE against the members table failed with
--     ERROR: column "member_id" does not exist
-- Recreated with an explicit USING *and* WITH CHECK on the correct column.
DROP POLICY IF EXISTS member_isolation_members ON lcc.members;
CREATE POLICY member_isolation_members ON lcc.members
    USING      (id::text = current_setting('app.current_member_id', true))
    WITH CHECK (id::text = current_setting('app.current_member_id', true));

COMMIT;
