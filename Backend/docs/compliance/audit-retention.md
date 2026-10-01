# Audit Retention

- All `lcc_audit.events` rows retained for **7 years**.
- Daily checksum chain verification by `audit-integrity-worker`.
- Any detected tamper → SEV-1 page.
