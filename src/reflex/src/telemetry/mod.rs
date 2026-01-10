use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::{layer::SubscriberExt, Registry};
pub mod forensics;
pub mod mirror;
pub mod decay;
pub mod metrics;

pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // 1. Create Resource (Identity)
    let resource = opentelemetry_sdk::Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", std::env::var("OTEL_SERVICE_NAME").unwrap_or("reflex_engine".into())),
        opentelemetry::KeyValue::new("service.namespace", "voltaire_reflex"),
    ]);

    // 2. Initialize Metrics Pipeline (The missing piece)
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_period(std::time::Duration::from_secs(3)) // Force 3s flush
        .with_resource(resource.clone())
        .build()?;

    global::set_meter_provider(meter_provider);

    // 3. Initialize Tracing Pipeline
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_resource(resource.clone())
                .with_sampler(opentelemetry_sdk::trace::Sampler::ParentBased(
                    Box::new(opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(1.0)), 
                ))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // 3.5 Initialize Logs Pipeline (New)
    let _logger = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    
    // Set global logger provider (Critical for the appender to find it)
    let log_layer = opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&opentelemetry::global::logger_provider());

    // 4. Connect to Tracing Subscriber
    // D-109: Add EnvFilter to control verbosity (Default: WARN for crates, DEBUG for reflex)
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn,reflex=debug"));

    let subscriber = Registry::default()
        .with(filter) // Add filter layer
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(log_layer)
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>)?;

    // info!("🔭 Telemetry Subsystems Initialized (Tracing + Metrics)");
    Ok(())
}

pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
    // MeterProvider doesn't need explicit shutdown in 0.20 usually, but if so:
    // global::shutdown_meter_provider(); // Not standard in all versions
}
