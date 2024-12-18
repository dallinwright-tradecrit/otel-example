use std::time::Duration;
use opentelemetry::{global, KeyValue};
use opentelemetry::trace::{TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace, Resource};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{Tracer, TracerProvider};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;

use tracing::{instrument};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use tracing_subscriber::util::SubscriberInitExt;

use opentelemetry_semantic_conventions::{
    attribute::{SERVICE_NAME, SERVICE_VERSION, DEPLOYMENT_ENVIRONMENT_NAME},
    SCHEMA_URL,
};

fn resource() -> Resource {
    Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "development"),
        ],
        SCHEMA_URL,
    )
}

fn init_tracer(service_name: String) -> TracerProvider {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter: opentelemetry_otlp::SpanExporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("grpc://localhost:4317")
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(Duration::from_secs(3))
        .build()
        .expect("Failed to create OTLP span exporter");

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_id_generator(trace::RandomIdGenerator::default())
        .with_max_attributes_per_span(16)
        .with_max_events_per_span(16)
        .with_max_links_per_span(16)
        .with_resource(resource())
        .build();

    global::set_tracer_provider(provider.clone());

    provider
}

// Construct MeterProvider for MetricsLayer
fn init_meter_provider(service_name: String) -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint("grpc://localhost:4317")
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
        .build()
        .unwrap();

    let reader = PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(std::time::Duration::from_secs(30))
        .build();

    // For debugging in development
    let stdout_reader = PeriodicReader::builder(
        opentelemetry_stdout::MetricExporter::default(),
        runtime::Tokio,
    )
        .build();

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource())
        .with_reader(reader)
        .with_reader(stdout_reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    meter_provider
}

fn init_logs(service_name: String) -> LoggerProvider {
    let exporter: opentelemetry_otlp::LogExporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint("grpc://localhost:4317")
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(std::time::Duration::from_secs(3))
        .build()
        .expect("Failed to create OTLP log exporter");

    LoggerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(resource())
        .build()
}

#[instrument(level = "info", name = "outer_child")]
fn test_print() {
    tracing::info!("Attempting outer_child test_print event");
    log::info!("Attempting outer_child test_print log");
    test_print_inner();
}

#[instrument(level = "info", name = "inner_child")]
fn test_print_inner() {
    tracing::info!("Attempting inner_child test_print event");
    log::info!("Attempting inner_child test_print log");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service_name = "sandbox".to_string();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_level(true)
        .with_ansi(true);

    let tracer_provider: TracerProvider = init_tracer(service_name.clone());
    let logger_provider: LoggerProvider = init_logs(service_name.clone());
    let meter_provider: SdkMeterProvider = init_meter_provider(service_name.clone());

    let tracer = tracer_provider.tracer("sandbox");
    let tracing_layer: OpenTelemetryLayer<Registry, Tracer> = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(fmt_layer)
        .init();

    test_print();

    // Shutdown pipelines
    tracer_provider.shutdown().expect("TracerProvider should shutdown successfully");
    logger_provider.shutdown().expect("LoggerProvider should shutdown successfully");
    meter_provider.shutdown().expect("MeterProvider should shutdown successfully");

    Ok(())
}
