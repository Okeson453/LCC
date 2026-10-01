# opportunity-intel

Discovery, enrichment, and φ (phi) scoring for opportunities.

## Endpoints

- `POST /internal/intelligence/discover` — produces candidate opportunities from cached signals.
- `POST /internal/intelligence/phi/score` — computes φ for a signal bundle.
- `GET /healthz`, `/readyz`.

## Modes

- **`rule_based`** — used when `labeled_sends < 200`. Deterministic scoring:
  `0.6 * decayed_signal + 0.3 * kb_fit + diversity_bonus`.
- **`ml`** — used when `labeled_sends ≥ 200`. Loads the model artifact from
  `LCC_MODEL_DIR/phi_model.json`. Returns the same `PhiResult` shape so the
  Compliance Governor can consume either.

## Components returned

- `signal_strength` (decayed, exponential decay with 14d half-life).
- `kb_fit` (matched KB facts / 5, capped at 1.0).
- `diversity_bonus` (0..0.10, scales with distinct signal kinds).

## Notes

- The `confidence` returned mirrors the Rust `PhiEvaluation::confidence` field.
- Both engines must emit identical JSON shapes for the Compliance Governor.
