//! Shared tracing initialization for test files.
//!
//! Provides OpenTelemetry SDK integration for real OTel span export
//! and JSON event logging for AI-readable trace analysis.
//!
//! Usage in tests:
//! ```rust,ignore
//! mod tracing_init;
//! tracing_init::init_otel("bootstrap");
//! // ... run compilation ...
//! tracing_init::shutdown_otel();
//! ```

use std::path::PathBuf;
use std::sync::Once;

// ---------------------------------------------------------------------------
// OpenTelemetry SDK mode — real OTel Tracer
// ---------------------------------------------------------------------------

/// Separate Once for OTel mode.
static OTEL_INIT: Once = Once::new();

/// Global storage for the TracerProvider so we can shut it down.
static OTEL_PROVIDER: std::sync::OnceLock<opentelemetry_sdk::trace::SdkTracerProvider> =
    std::sync::OnceLock::new();

/// Output path for the OTel trace file.
fn otel_trace_path(label: &str) -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join(format!("otel-trace-{}.jsonl", label))
}

/// Initialize tracing with real OpenTelemetry SDK.
///
/// Sets up:
/// - **SdkTracerProvider** with stdout exporter → writes OTel spans to
///   `otel-trace-<label>.jsonl` (OTel span format with trace IDs, parent IDs)
/// - **tracing-opentelemetry layer** bridges `tracing` spans → OTel spans
/// - **JSON fmt layer** writes tracing events to `otel-trace-<label>.events.jsonl`
///   (AI-readable: timestamp, level, fields, span context)
/// - **Console fmt layer** for human-readable output
///
/// Usage:
/// ```rust,ignore
/// mod tracing_init;
/// tracing_init::init_otel("bootstrap");
/// // ... run compilation ...
/// tracing_init::shutdown_otel();
/// ```
pub fn init_otel(label: &str) {
    OTEL_INIT.call_once(|| {
        use opentelemetry::trace::TracerProvider as _;
        use tracing_opentelemetry::OpenTelemetryLayer;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let trace_path = otel_trace_path(label);

        // --- OTel Tracer Provider with stdout exporter ---
        let exporter = opentelemetry_stdout::SpanExporter::default();
        let resource = opentelemetry_sdk::Resource::builder()
            .with_service_name("sasspile")
            .build();
        let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_simple_exporter(exporter)
            .with_resource(resource)
            .build();

        let tracer = tracer_provider.tracer("sasspile");

        // --- OTel Layer: bridges tracing spans → OTel spans ---
        let otel_layer = OpenTelemetryLayer::new(tracer);

        // --- JSON layer for AI-readable tracing events ---
        let events_path = trace_path.with_extension("events.jsonl");
        let events_file = std::fs::File::create(&events_path)
            .unwrap_or_else(|e| panic!("failed to create {}: {}", events_path.display(), e));
        let json_writer = std::sync::Mutex::new(events_file);

        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(json_writer)
            .with_target(true);

        // --- Console layer ---
        let console_layer = tracing_subscriber::fmt::layer()
            .with_target(false);

        // --- Filter ---
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

        tracing_subscriber::registry()
            .with(filter)
            .with(console_layer)
            .with(json_layer)
            .with(otel_layer)
            .init();

        // Store provider for shutdown
        OTEL_PROVIDER.set(tracer_provider).ok();

        tracing::info!(
            stage = "tracing_init",
            otel_trace_file = %trace_path.display(),
            otel_events_file = %events_path.display(),
            "OpenTelemetry SDK initialized (real OTel spans + JSON events)"
        );
    });
}

/// Shutdown the OTel TracerProvider, flushing all pending spans.
pub fn shutdown_otel() {
    if let Some(provider) = OTEL_PROVIDER.get() {
        let _ = provider.force_flush();
        let _ = provider.shutdown();
        tracing::info!(stage = "tracing_init", "OTel SDK shut down, spans flushed");
    }
}
