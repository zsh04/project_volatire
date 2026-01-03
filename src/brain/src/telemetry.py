import os
from opentelemetry import trace, metrics
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.exporter.otlp.proto.grpc.metric_exporter import OTLPMetricExporter
from opentelemetry.instrumentation.grpc import GrpcInstrumentorServer


from opentelemetry.sdk._logs import LoggerProvider, LoggingHandler
from opentelemetry.sdk._logs.export import BatchLogRecordProcessor
from opentelemetry.exporter.otlp.proto.grpc._log_exporter import OTLPLogExporter
import logging


def init_telemetry():
    """
    Initializes OpenTelemetry for the Brain service.
    Configures Tracing, Metrics, and Logs to send to the OTel Collector via gRPC.
    """
    service_name = os.getenv("OTEL_SERVICE_NAME", "voltaire_brain")
    if service_name == "reflex_engine":
        service_name = "voltaire_brain"

    endpoint = os.getenv("OTEL_EXPORTER_OTLP_ENDPOINT", "127.0.0.1:4317")
    if endpoint.startswith("http://"):
        endpoint = endpoint.replace("http://", "")
    if endpoint.startswith("https://"):
        endpoint = endpoint.replace("https://", "")

    print(f"🔭 Initializing Telemetry for {service_name} -> {endpoint}")

    resource = Resource.create(
        {
            "service.name": service_name,
            "service.namespace": "voltaire_brain",
        }
    )

    # --- Tracing ---
    # Smart Sampling: Push logic left to reduce local CPU/Net overhead
    from opentelemetry.sdk.trace.sampling import ParentBased, TraceIdRatioBased

    # Default to 100% sampling if not set, handled by Collector
    sample_rate = float(os.getenv("OTEL_TRACES_SAMPLER_ARG", "1.0"))
    sampler = ParentBased(root=TraceIdRatioBased(sample_rate))

    trace_provider = TracerProvider(resource=resource, sampler=sampler)
    span_exporter = OTLPSpanExporter(endpoint=endpoint, insecure=True)
    span_processor = BatchSpanProcessor(span_exporter)
    trace_provider.add_span_processor(span_processor)
    trace.set_tracer_provider(trace_provider)

    # --- Metrics ---
    metric_reader = PeriodicExportingMetricReader(
        OTLPMetricExporter(endpoint=endpoint, insecure=True),
        export_interval_millis=5000,
    )
    meter_provider = MeterProvider(resource=resource, metric_readers=[metric_reader])
    metrics.set_meter_provider(meter_provider)

    # --- Instrumentation ---
    # Auto-instrument gRPC server
    grpc_server_instrumentor = GrpcInstrumentorServer()
    grpc_server_instrumentor.instrument()

    # --- Logs ---
    logger_provider = LoggerProvider(resource=resource)
    log_exporter = OTLPLogExporter(endpoint=endpoint, insecure=True)
    logger_provider.add_log_record_processor(BatchLogRecordProcessor(log_exporter))

    # Attach OTel handler to root logger
    otel_handler = LoggingHandler(level=logging.INFO, logger_provider=logger_provider)
    logging.getLogger().addHandler(otel_handler)

    return trace_provider, meter_provider, logger_provider
