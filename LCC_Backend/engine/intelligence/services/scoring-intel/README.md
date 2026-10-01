# scoring-intel

ρ (rho), ab_d, h_c model inference.

## Endpoints

- `POST /internal/intelligence/scoring/rho/score` — ranks a contact for sequence selection.
- `POST /internal/intelligence/scoring/abd/update` — adaptive daily-pacing multiplier.
- `POST /internal/intelligence/scoring/hc/compute` — composite account health score.
- `GET /healthz`, `/readyz`.

## ρ

- Rule-based (labeled_sends < 200): VIP weight 0.35, recency 0.30, mutual 0.10, kb_fit 0.15, response_history 0.10.
- ML (labeled_sends ≥ 200): loads `rho_model.json` artifact from `LCC_MODEL_DIR`.

## ab_d

Adaptive multiplier ∈ [0.5, 2.0] based on:
- observed acceptance rate (< 20% → ×0.7)
- observed error rate (> 10% → ×0.8)
- restriction events (> 0 → ×0.6)

## h_c

Composite of 5 weighted components → band:
- WARMUP ≥ 0.85
- STANDARD ≥ 0.65
- COLD otherwise
