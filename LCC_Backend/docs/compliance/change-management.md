# Compliance Config Change Management

Per Backend Design Concept §47, every `ccfg-*` activation is gated by
**two compliance reviewers** in addition to the activator. The activation
row records all three names in `two_reviewer_signed_by` and
`activated_by`.

Activation flow:
1. Author uploads new `ccfg-<version>.yaml`.
2. Reviewer A approves (recorded in metadata).
3. Reviewer B approves (recorded in metadata).
4. Activator calls `lcc compliance activate <version>`.
5. The Compliance Governor's `config_version` field flips to the new
   version. Old in-flight permits continue to be valid for their 60s TTL.
