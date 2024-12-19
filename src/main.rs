

use telemetry_rust::TelemetryProvider;
use telemetry_rust::TelemetryProviderConfig;
use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetryLayer;

use opentelemetry_semantic_conventions::{
    attribute::SERVICE_NAME,
    SCHEMA_URL,
};

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
    let resource: Resource = Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, "test-service"),
        ],
        SCHEMA_URL,
    );

    let telemetry_provider_config: TelemetryProviderConfig = TelemetryProviderConfig {
        resource: resource.clone(),
        trace_url: "grpc://localhost:4317".to_string(),
        log_url: "grpc://localhost:4317".to_string(),
        metric_url: "grpc://localhost:4317".to_string(),
    };

    let telemetry_provider = TelemetryProvider::new(telemetry_provider_config);

    test_print();

    // Shutdown pipelines
    telemetry_provider.shutdown();

    Ok(())
}
