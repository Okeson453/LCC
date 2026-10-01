"""opportunity-intel — discovery, enrichment, φ scoring.

The φ model (receptiveness) is the Python mirror of `crates/compliance/src/phi.rs`.
It returns a phi_score in [0, 1] per opportunity. When labeled sends < 200,
the Rust service falls back to a rule-based variant; the Python wrapper
mirrors that fallback explicitly.
"""

__version__ = "0.1.0"
