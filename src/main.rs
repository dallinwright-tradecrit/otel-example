use std::time::Duration;
use opentelemetry::trace::{TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{Tracer, TracerProvider};
use tracing::{span};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;


fn init_tracer(service_name: &String) -> TracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("grpc://localhost:4317")
        .with_protocol(opentelemetry_otlp::Protocol::Grpc)
        .with_timeout(Duration::from_secs(3))
        .build()
        .expect("Failed to create OTLP trace exporter");

    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", service_name.clone()),
        ]))
        .build();

    provider
}

#[tokio::main]
async fn main() {
    let service_name = "sandbox".to_string();

    let tracer_provider = init_tracer(&service_name);

    let tracer: Tracer = tracer_provider.tracer("main");

    // Create a tracing layer with the configured tracer
    let telemetry: OpenTelemetryLayer<Registry, Tracer> = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_level(true)
        .with_ansi(true);

    let subscriber = Registry::default()
        .with(telemetry)
        .with(fmt_layer);

    tracing::subscriber::with_default(subscriber, || {
        // Spans will be sent to the configured OpenTelemetry exporter
        let root = span!(tracing::Level::INFO, "app_start", work_units = 2);
        let _enter = root.enter();

        tracing::info!("This event will be logged in the root span.");
    });
}
