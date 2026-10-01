-- 0005_kb.sql
-- Professional Knowledge Base — records and chunks.

BEGIN;

CREATE TYPE lcc.kb_category AS ENUM (
    'achievements', 'skills', 'projects', 'experience',
    'voice_samples', 'case_studies', 'testimonials', 'preferences'
);

CREATE TABLE lcc.kb_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    category lcc.kb_category NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL DEFAULT '',
    source_kind TEXT NOT NULL DEFAULT 'manual',
    source_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
CREATE INDEX idx_kb_records_member ON lcc.kb_records(member_id);
SELECT lcc.attach_member_rls('kb_records');

CREATE TABLE lcc.kb_record_chunks (
    id TEXT PRIMARY KEY,                  -- "{record_id}:{chunk_index}"
    record_id UUID NOT NULL REFERENCES lcc.kb_records(id) ON DELETE CASCADE,
    member_id UUID NOT NULL REFERENCES lcc.members(id) ON DELETE CASCADE,
    chunk_index INT NOT NULL,
    chunk_count INT NOT NULL,
    text TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    embedding_model_version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_kb_chunks_record ON lcc.kb_record_chunks(record_id);
SELECT lcc.attach_member_rls('kb_record_chunks');

COMMIT;
