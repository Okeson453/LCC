# Web manifests — D-04 rationale

Two manifests co-exist intentionally:

- `manifest.webmanifest` — PWA spec target (referenced by `<link rel="manifest">`)
- `manifest.json` — Android Chrome, used at install prompt

Content is identical (kept in sync by `tools/scripts/sync-manifests.ts` if present).
The original audit flagged the duplication as D-04. We kept both because a
single file with `manifest` value= doesn't satisfy both `link rel="manifest"`
and Android's legacy `manifest.json` install path simultaneously.
