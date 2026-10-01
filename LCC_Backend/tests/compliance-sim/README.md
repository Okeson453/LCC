# Compliance Simulation

`sim_strict_mode.py` runs synthetic actions through the Compliance Governor
to verify guard behavior end-to-end. Cases cover:

- First-time success (allow)
- Cooldown violation (deny)
- Daily-cap overflow (deny)
- Low grounding (deny)
- Low h_c (deny)

In CI: runs against the deployed governor in the staging environment
and asserts all cases match the expected decision.
