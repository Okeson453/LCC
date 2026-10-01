# Vector DB Schema — OKESON-LCC

Qdrant is the primary vector store.

## Collections

### `kb_records` — Professional Knowledge Base
- **Dimensions**: 384 (all-MiniLM-L6-v2 default; configurable per deployment).
- **Distance**: Cosine.
- **Point ID**: `"{record_id}:{chunk_index}"` (string).
- **Payload**:
  ```json
  {
    "record_id": "uuid",
    "member_id": "uuid",
    "category": "achievements|skills|projects|experience|voice_samples|case_studies|testimonials|preferences",
    "title": "string",
    "chunk_index": 0,
    "chunk_count": 3,
    "text": "string",
    "content_hash": "sha256",
    "source_kind": "manual|linkedin|document_upload|external_import",
    "source_url": "string",
    "embedding_model_version": "all-MiniLM-L6-v2@1",
    "created_at": "ISO-8601"
  }
  ```

### `voice_samples` — Approved past posts for tone matching
- **Dimensions**: 384.
- **Distance**: Cosine.
- **Point ID**: UUID.
- **Payload**:
  ```json
  {
    "member_id": "uuid",
    "text": "string",
    "source": "approved_post",
    "source_url": "string",
    "fingerprint": { "avg_sentence_length": ..., "tone_descriptor": "punchy" },
    "created_at": "ISO-8601"
  }
  ```

### `opportunity_signals` — Embeddings of signal text for similarity search
- **Dimensions**: 384.
- **Distance**: Cosine.

### `profile_snapshots` — Profile data embeddings (for similarity across snapshots)
- **Dimensions**: 384.
- **Distance**: Cosine.

## Indexing

Qdrant HNSW (default `m=16, ef_construct=100`).

## Operations

- **Search**: top-K with payload filter `{member_id, category}`.
- **Upsert**: idempotent on point ID.
- **Delete**: by point ID (cascade-delete from kb_record_chunks via worker).
- **Dedup**: pre-ingest search with `score >= 0.92` → return existing record id.

## Capacity & TTL

- No automatic TTL on vectors; third-party-sourced data is stamped with
  `ttl_expires_at` in the SQL `profile_snapshots` / `opportunity_signals`
  tables. The `data-purge-worker` removes expired rows and their vectors.
