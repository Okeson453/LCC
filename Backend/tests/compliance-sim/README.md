# Compliance Simulation

`simulator/main.py` runs the scenarios in `inputs/` against a **live**
`compliance-governor` and fails when a decision disagrees with the scenario's
expectation.

Each scenario names the decision it expects and, optionally, the guard that must
produce it, so a deny for the wrong reason is a failure:

| Scenario | Expectation |
|---|---|
| `permit_grounded_post` | PERMIT — grounded post, healthy account |
| `deny_grounding_missing` | DENY — `grounding` guard |
| `deny_account_health_low_h_c` | DENY — `account_health` guard (H_c below the tier floor) |
| `deny_restriction_flag` | DENY — `restriction_flag` guard |
| `deny_approval_state_missing` | DENY — `approval_state` guard |

## Running it

The governor has to be up, with a migrated database and Redis:

```bash
# from Backend/
export DATABASE_URL=postgres://lcc:lcc@127.0.0.1:5432/lcc_test
export REDIS_URL=redis://127.0.0.1:6379
export COMPLIANCE_CONFIG_PATH=config/compliance/ccfg-2025-01-01-rc1.yaml
export LCC_AUTH_JWT_SECRET=local-dev-only-jwt-secret-32-bytes
export PERMIT_SIGNING_SEED=0000000000000000000000000000000000000000000000000000000000000001
cargo build -p compliance-governor --bin compliance-governor
./target/debug/compliance-governor &

export COMPLIANCE_GOVERNOR_URL=http://127.0.0.1:8080
PYTHONPATH=packages/compliance-types/src python tests/compliance-sim/simulator/main.py
```

Reports land in `reports/` (generated, not committed) and the CI job uploads
them as an artifact.

## In CI

`.github/workflows/compliance-evals.yaml` starts Postgres and Redis, applies the
real migrations, builds and starts the governor, asserts that `/healthz`,
`/readyz` and `/metrics` are reachable without credentials while the admin route
is not, then runs every scenario. Any mismatch fails the job.

`sim_strict_mode.py` is an older single-file stub that prints `"actual": "stub"`
for its cases; it is not the gate and does not evaluate anything.
