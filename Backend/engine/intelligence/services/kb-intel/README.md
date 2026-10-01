# kb-intel

KB CRUD + chunking + duplicate detection + retrieval.

## Endpoints

- `POST /internal/intelligence/kb/ingest` — chunk, embed, dedup, upsert.
- `POST /internal/intelligence/kb/search` — vector search with member + category filter.
- `POST /internal/intelligence/kb/dedup-check` — check for duplicates before ingest.
- `GET /healthz`, `/readyz`.

## Pipeline

```
text → chunk(max_chars=1200, overlap=200)
     → embed (sentence-transformers)
     → dedup check (qdrant search, threshold=0.92)
     → upsert KB records (one row per chunk)
     → publish kb.record.created
```
