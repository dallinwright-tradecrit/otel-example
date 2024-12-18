// use std::time::Duration;
// use opentelemetry::{global, KeyValue};
// use opentelemetry_sdk::{runtime, trace, Resource};
//
// use opentelemetry_sdk::logs::{LoggerProvider};
// use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
// use opentelemetry_sdk::propagation::TraceContextPropagator;
// use opentelemetry_sdk::trace::{Tracer, TracerProvider};
// use opentelemetry::trace::{Tracer as _, TracerProvider as _};
// use opentelemetry_otlp::WithExportConfig;
// use tracing::{span, Level};
// use tracing_subscriber::fmt::writer::MakeWriterExt;
// use tracing_subscriber::layer::SubscriberExt;
// use tracing_subscriber::util::SubscriberInitExt;
// use crate::error::Error;
//
// fn init_tracer() -> Result<(), Error> {
//     global::set_text_map_propagator(TraceContextPropagator::new());
//
//     let exporter = opentelemetry_otlp::SpanExporter::builder()
//         .with_tonic()
//         .with_endpoint("localhost:4317")
//         .with_timeout(Duration::from_secs(3))
//         .with_protocol(opentelemetry_otlp::Protocol::Grpc)
//         .build()
//         .expect("Failed to create OTLP trace exporter");
//
//     let provider = TracerProvider::builder()
//         // .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
//         .with_simple_exporter(exporter)
//         .with_id_generator(trace::RandomIdGenerator::default())
//         .with_max_attributes_per_span(16)
//         .with_max_events_per_span(16)
//         .with_max_links_per_span(16)
//         .with_resource(Resource::new(vec![KeyValue::new("service.name", "sandbox")]))
//         .build();
//
//     global::set_tracer_provider(provider);
//
//     Ok(())
// }
//
// pub fn init_observability() -> Result<(), Error> {
//     global::set_text_map_propagator(TraceContextPropagator::new());
//
//     init_tracer()?;
//
//     let fmt_layer = tracing_subscriber::fmt::layer()
//         .with_target(true)
//         .with_thread_ids(true)
//         .with_thread_names(true)
//         .with_timer(tracing_subscriber::fmt::time::ChronoUtc::rfc_3339())
//         .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
//         .with_level(true)
//         .with_ansi(true);
//
//     tracing_subscriber::registry()
//         .with(fmt_layer)
//         .init();
//
//     tracing::info!("Observability initialized");
//
//     Ok(())
// }
