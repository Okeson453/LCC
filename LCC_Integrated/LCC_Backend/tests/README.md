# Tests

- `contract/` — JSON-Schema + proto contract validation.
- `integration/` — service-to-service flows (require running services).
- `load/` — Locust-based load tests (Governor p99 < 200ms, etc.).
- `compliance-sim/` — synthetic compliance simulations.

Run with:

```bash
just test         # unit + contract
just test-int     # integration
just test-load    # load (Locust)
just test-comp    # compliance sim
```
