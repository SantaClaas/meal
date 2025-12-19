use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::{SCHEMA_URL, attribute};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub(super) fn setup() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // To debug axum extractor rejections see https://docs.rs/axum/latest/axum/extract/index.html#logging-rejections

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        // .with_http()
        .build()?;

    let resource = Resource::builder()
        .with_service_name(env!("CARGO_PKG_NAME"))
        //TODO I don't understand the difference between setting this as an attribute or as a resource
        .with_attributes([
            KeyValue::new(attribute::SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(attribute::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        ])
        .with_schema_url(
            [
                KeyValue::new(attribute::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(attribute::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            ],
            SCHEMA_URL,
        )
        .build();

    let trace_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    let tracer = trace_provider.tracer(format!("{}-tracer", env!("CARGO_PKG_NAME")));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    Ok(())
}
