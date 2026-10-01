-- 0006_content.sql
-- Content items (drafts, scheduled, published), approvals, post metrics.

BEGIN;

CREATE TYPE lcc.content_kind AS ENUM (
    'post', 'comment', 'reply', 'dm', 'connection_request',
    'sequence_step', 'article', 'video', 'poll', 'document',
    'profile_edit', 'client_proposal'
);

CREATE TYPE lcc.content_state AS ENUM (
    'drafting', 'quality_check', 'pending_approval',
    'scheduled', 'publishing', 'published', 'failed',
    'rejected', 'archived'
);

CREATE TABLE lcc.content_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    kind lcc.content_kind NOT NULL,
    state lcc.content_state NOT NULL DEFAULT 'drafting',
    body TEXT NOT NULL,
    media JSONB,                  -- {type, url, alt_text}
    pillar TEXT,
    scheduled_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    linkedin_post_id TEXT,
    kb_refs UUID[] NOT NULL DEFAULT '{}'::UUID[],
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_error TEXT
);
CREATE INDEX idx_content_items_member ON lcc.content_items(member_id);
CREATE INDEX idx_content_items_state ON lcc.content_items(state) WHERE state IN ('scheduled', 'publishing');
SELECT lcc.attach_member_rls('content_items');

CREATE TABLE lcc.post_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content_item_id UUID NOT NULL REFERENCES lcc.content_items(id) ON DELETE CASCADE,
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    impressions INT NOT NULL DEFAULT 0,
    reactions INT NOT NULL DEFAULT 0,
    comments INT NOT NULL DEFAULT 0,
    shares INT NOT NULL DEFAULT 0,
    engagement_rate NUMERIC(5, 4),
    snapshot_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_post_metrics_content ON lcc.post_metrics(content_item_id);
SELECT lcc.attach_member_rls('post_metrics');

COMMIT;
