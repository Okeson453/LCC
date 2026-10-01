"""PermitClaims — Python mirror of Rust `crates/compliance::permit_token`.

Used by services that need to verify a permit_token (typically the Integration
Gateway, but only in Python; the production verifier is in Rust).
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime


@dataclass(frozen=True)
class PermitClaims:
    """JWT claims of an issued permit_token."""

    iss: str                 # "compliance-governor"
    aud: str                 # "integration-gateway"
    sub: str                 # member_id
    act: str                 # action_id
    typ: str                 # ActionType as string
    risk_tier: int
    approval_id: str
    config_version: str
    iat: int                 # unix seconds
    exp: int                 # unix seconds
    jti: str

    @property
    def issued_at(self) -> datetime:
        return datetime.fromtimestamp(self.iat, tz=UTC)

    @property
    def expires_at(self) -> datetime:
        return datetime.fromtimestamp(self.exp, tz=UTC)

    def is_expired(self, now: datetime | None = None) -> bool:
        ts = (now or datetime.now(UTC)).timestamp()
        return ts >= self.exp
