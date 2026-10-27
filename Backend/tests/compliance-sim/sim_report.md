# Compliance Simulation Report

Results are no longer maintained by hand. Every run writes
`reports/sim_report_<timestamp>.json` (total / passed / failed plus the guard
each denial came from), and the `Compliance Evals` workflow publishes that
directory as the `compliance-sim-report` artifact.

The previous table here claimed pass results dated 2025-01-15 for cases that the
old simulator never actually evaluated against a governor.
