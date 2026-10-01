"""Permit token integration test."""

import os
import pytest

GOVERNOR_URL = os.environ.get("COMPLIANCE_GOVERNOR_URL", "http://localhost:18080")
GATEWAY_URL = os.environ.get("INTEGRATION_GATEWAY_URL", "http://localhost:18081")


@pytest.mark.integration
class TestPermitToken:
    def test_valid_token_accepted(self):
        pytest.skip("requires running governor + gateway")

    def test_expired_token_denied(self):
        pytest.skip("requires running governor + gateway")

    def test_wrong_audience_denied(self):
        pytest.skip("requires running governor + gateway")
