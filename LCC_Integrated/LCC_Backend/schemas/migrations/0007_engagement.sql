-- 0007_engagement.sql
-- Inbound messages, engagement replies, sequences.

BEGIN;

CREATE TYPE lcc.inbound_kind AS ENUM ('comment', 'dm', 'mention', 'connection_request_inbound', 'post_reaction');

CREATE TABLE lcc.inbound_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    contact_id UUID,                 -- FK to contacts (added later)
    kind lcc.inbound_kind NOT NULL,
    body TEXT NOT NULL,
    linkedin_message_id TEXT UNIQUE,
    thread_id TEXT,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    read_at TIMESTAMPTZ,
    is_archived BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB
);
CREATE INDEX idx_inbound_member ON lcc.inbound_messages(member_id, received_at DESC);
SELECT lcc.attach_member_rls('inbound_messages');

CREATE TABLE lcc.engagement_replies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    inbound_message_id UUID REFERENCES lcc.inbound_messages(id) ON DELETE SET NULL,
    content_item_id UUID REFERENCES lcc.content_items(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'drafted',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
SELECT lcc.attach_member_rls('engagement_replies');

CREATE TYPE lcc.sequence_state AS ENUM ('active', 'paused', 'completed', 'abandoned', 'replied');

CREATE TABLE lcc.sequences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL,
    template_id UUID,
    state lcc.sequence_state NOT NULL DEFAULT 'active',
    current_step INT NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    paused_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);
CREATE INDEX idx_sequences_member ON lcc.sequences(member_id);
SELECT lcc.attach_member_rls('sequences');

CREATE TABLE lcc.sequence_steps (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sequence_id UUID NOT NULL REFERENCES lcc.sequences(id) ON DELETE CASCADE,
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    step_index INT NOT NULL,
    body TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    scheduled_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    response_at TIMESTAMPTZ,
    skip_reason TEXT
);
CREATE INDEX idx_sequence_steps_seq ON lcc.sequence_steps(sequence_id, step_index);
SELECT lcc.attach_member_rls('sequence_steps');

COMMIT;
