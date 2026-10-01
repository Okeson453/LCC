# compliance-governor

The **single arbiter** of every externally-visible LinkedIn action. Implements
the 8 sequential guards from `LinkedIn_Manager_Technical_Design_Spec` §11.

## Endpoints

- `POST /internal/governor/evaluate` — synchronous inline guard evaluation.
- `POST /internal/compliance/config/propose` — propose a new compliance config.
- `POST /internal/compliance/config/review` — review a proposal (one of two).
- `POST /internal/compliance/config/activate` — activate (requires two distinct signers).
- `GET  /internal/compliance/config/list` — list active + historical versions.
- `GET  /healthz` / `GET /readyz` / `GET /metrics`.

## SLOs

- p99 guard-stack evaluation: **< 200ms**
- Availability: **≥ 99.95%**
- Sync inline (no fire-and-forget).

## Non-negotiables enforced

- §2: only this service issues `permit_token` (HSM-signed JWT).
- §3: all 8 guards additive; partial-pass = deny.
- §4: numeric constants are versioned config with two-reviewer sign-off.
- §11: synchronous inline evaluation; no async variant.
- §12: append-only audit log via `audit-client`.

## Local dev

```bash
just bootstrap          # local stack
just rust-build         # build
just proto-gen          # regenerate proto stubs
cargo run -p compliance-governor
```
