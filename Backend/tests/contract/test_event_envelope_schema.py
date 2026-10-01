"""Event envelope JSON-Schema contract test."""

import json
import pytest
from pathlib import Path

ENVELOPE_SCHEMA = json.loads(
    (Path(__file__).parent.parent.parent / "schemas/events/envelope.schema.json").read_text()
)

EVENT_SCHEMAS_DIR = Path(__file__).parent.parent.parent / "schemas/events"


@pytest.mark.contract
class TestEventEnvelope:
    def test_envelope_valid_minimal(self):
        # Sanity: a sample envelope matches the schema
        try:
            import jsonschema
        except ImportError:
            pytest.skip("jsonschema not installed")
        envelope = {
            "event_id": "00000000-0000-0000-0000-000000000001",
            "topic": "member.created",
            "trace_id": "trace-1",
            "producer_service": "identity-svc",
            "occurred_at": "2025-01-15T10:00:00Z",
            "idempotency_key": "member.created:00000000-0000-0000-0000-000000000001",
            "schema_version": 1
        }
        jsonschema.validate(instance={"header": envelope, "payload": {}}, schema=ENVELOPE_SCHEMA)

    def test_all_event_schemas_reference_envelope(self):
        for path in EVENT_SCHEMAS_DIR.glob("*.schema.json"):
            if path.name == "envelope.schema.json":
                continue
            data = json.loads(path.read_text())
            header_ref = data.get("properties", {}).get("header", {}).get("$ref", "")
            assert header_ref == "envelope.schema.json", f"{path.name} header ref is {header_ref!r}"

    def test_all_event_schemas_have_header_and_payload(self):
        for path in EVENT_SCHEMAS_DIR.glob("*.schema.json"):
            if path.name == "envelope.schema.json":
                continue
            data = json.loads(path.read_text())
            required = data.get("required", [])
            assert "header" in required, f"{path.name} missing 'header' required"
            assert "payload" in required, f"{path.name} missing 'payload' required"
