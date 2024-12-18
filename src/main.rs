use std::time::Duration;
use opentelemetry::{global, KeyValue};
use opentelemetry::trace::{TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{trace, Resource};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{Tracer, TracerProvider};

use tracing::{instrument, subscriber};
use tracing_log::LogTracer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use tracing_subscriber::util::SubscriberInitExt;

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
        .with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", service_name),
        ]))
        .build();

    global::set_tracer_provider(provider.clone());

    provider
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
        .with_resource(Resource::new(vec![KeyValue::new("service.name", service_name)]))
        .build()
}

#[instrument(level = "info", name = "outer_child")]
fn test_print() {
    tracing::info!("Attempting outer_child test_print event");
    log::trace!("Attempting outer_child test_print log");
    test_print_inner();
}

#[instrument(level = "info", name = "inner_child")]
fn test_print_inner() {
    tracing::info!("Attempting inner_child test_print event");
    log::trace!("Attempting inner_child test_print log");
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

    let tracer = tracer_provider.tracer("sandbox");

    let tracing_layer: OpenTelemetryLayer<Registry, Tracer> = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = tracing_subscriber::registry()
        .with(tracing_layer)
        .with(fmt_layer)
        .init();

    test_print();

    // Shutdown trace pipeline
    tracer_provider.shutdown().expect("TracerProvider should shutdown successfully");

    Ok(())
}
