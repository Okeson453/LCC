"""OpenTelemetry initialization helper."""

from __future__ import annotations

import os

from intelligence_common.logging import get_logger

log = get_logger(__name__)


def init_tracing(service_name: str, otlp_endpoint: str | None = None) -> None:
    """Initialize OpenTelemetry tracing for the service.

    No-op if opentelemetry packages aren't installed (we don't want to hard-fail
    services that don't have OTel installed).
    """
    try:
        from opentelemetry import trace
        from opentelemetry.sdk.resources import Resource
        from opentelemetry.sdk.trace import TracerProvider
        from opentelemetry.sdk.trace.export import BatchSpanProcessor
    except ImportError:
        log.info("otel_not_installed_skipping_tracing")
        return

    endpoint = otlp_endpoint or os.environ.get("OTLP_ENDPOINT")
    if not endpoint:
        log.info("otel_no_endpoint_skipping_export")
        return

    try:
        from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
        resource = Resource.create({"service.name": service_name})
        provider = TracerProvider(resource=resource)
        exporter = OTLPSpanExporter(endpoint=endpoint, insecure=True)
        provider.add_span_processor(BatchSpanProcessor(exporter))
        trace.set_tracer_provider(provider)
        log.info("otel_tracing_initialized", endpoint=endpoint)
    except Exception as e:
        log.warning("otel_init_failed", error=str(e))


def init_metrics(service_name: str, otlp_endpoint: str | None = None) -> None:
    """Initialize OpenTelemetry metrics."""
    try:
        from opentelemetry import metrics
        from opentelemetry.sdk.metrics import MeterProvider
        from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
        from opentelemetry.sdk.resources import Resource
    except ImportError:
        return

    endpoint = otlp_endpoint or os.environ.get("OTLP_ENDPOINT")
    if not endpoint:
        return

    try:
        from opentelemetry.exporter.otlp.proto.grpc.metric_exporter import OTLPMetricExporter
        resource = Resource.create({"service.name": service_name})
        reader = PeriodicExportingMetricReader(OTLPMetricExporter(endpoint=endpoint, insecure=True))
        provider = MeterProvider(resource=resource, metric_readers=[reader])
        metrics.set_meter_provider(provider)
    except Exception:
        return
