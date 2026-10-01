-- 0008_network_crm.sql
-- Contacts, network graph, list memberships, tags.

BEGIN;

CREATE TYPE lcc.relationship_strength AS ENUM ('none', 'weak', 'medium', 'strong', 'strong_recent');

CREATE TABLE lcc.contacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    linkedin_id TEXT,                            -- NULL for non-LinkedIn contacts
    display_name TEXT NOT NULL,
    headline TEXT,
    company TEXT,
    title TEXT,
    is_vip BOOLEAN NOT NULL DEFAULT FALSE,
    is_mutual BOOLEAN NOT NULL DEFAULT FALSE,
    tags TEXT[] NOT NULL DEFAULT '{}',
    relationship_strength lcc.relationship_strength NOT NULL DEFAULT 'none',
    last_contact_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_contacts_member ON lcc.contacts(member_id);
CREATE INDEX idx_contacts_linkedin ON lcc.contacts(linkedin_id) WHERE linkedin_id IS NOT NULL;
SELECT lcc.attach_member_rls('contacts');

CREATE TABLE lcc.network_lists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    list_kind TEXT NOT NULL DEFAULT 'manual',
    is_dynamic BOOLEAN NOT NULL DEFAULT FALSE,
    rule JSONB,                                  -- dynamic list query spec
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (member_id, name)
);
SELECT lcc.attach_member_rls('network_lists');

CREATE TABLE lcc.network_list_memberships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    list_id UUID NOT NULL REFERENCES lcc.network_lists(id) ON DELETE CASCADE,
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES lcc.contacts(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (list_id, contact_id)
);
CREATE INDEX idx_list_memberships_contact ON lcc.network_list_memberships(contact_id);
SELECT lcc.attach_member_rls('network_list_memberships');

CREATE TABLE lcc.tags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT,
    UNIQUE (member_id, name)
);
SELECT lcc.attach_member_rls('tags');

COMMIT;
