# store-assets

Submission assets for Chrome Web Store / Firefox AMO.

Required at submission time:
- manifest.json (already in `public/`)
- icons (16, 32, 48, 128, 256) — see `public/icons/` (placeholders until the design team finalizes)
- screenshots: 1280×800 and 640×400
- privacy disclosures (data usage, OAuth scope, no LinkedIn-side credentials)
- store-assets/screenshots/ contains 2 placeholder PNG slots

The placeholder PNGs are intentionally missing here so contributors see the gap
and create them. Build script: `pnpm tools/scripts/render-store-screenshots.sh`.
