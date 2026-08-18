//! Shared tracing initialization for test files.
//!
//! Supports two modes:
//! - **Console only** (default): formatted output to stderr
//! - **Flamegraph mode**: when `SASSPILE_FLAME=1` is set, also writes
//!   a flamegraph to `flame-<test_name>.svg` in the project root.
//!
//! Usage in tests:
//! ```rust,ignore
//! mod tracing_init;
//! tracing_init::init("bootstrap");
//! ```

use std::path::PathBuf;
use std::sync::Once;

static INIT: Once = Once::new();

/// Global storage for the flame guard so it can be flushed on demand.
static FLAME_GUARD: std::sync::OnceLock<
    std::sync::Mutex<Option<tracing_flame::FlushGuard<std::io::BufWriter<std::fs::File>>>>,
> = std::sync::OnceLock::new();

/// Initialize tracing for tests.
///
/// When the `SASSPILE_FLAME` environment variable is set to "1",
/// a flamegraph file (`flame-<label>.svg`) will be generated in
/// the project root directory.
///
/// Otherwise, standard formatted console output is used.
pub fn init(label: &str) {
    INIT.call_once(|| {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

        if std::env::var("SASSPILE_FLAME").unwrap_or_default() == "1" {
            // Flamegraph mode: write to file + console
            let flame_path = flame_path(label);

            let (flame_layer, flame_guard) =
                tracing_flame::FlameLayer::with_file(&flame_path)
                    .expect("failed to create flame layer");

            // Store guard in global so we can drop it to flush
            let guard_storage = FLAME_GUARD
                .get_or_init(|| std::sync::Mutex::new(None));
            *guard_storage.lock().unwrap() = Some(flame_guard);

            use tracing_subscriber::layer::SubscriberExt;
            use tracing_subscriber::util::SubscriberInitExt;

            tracing_subscriber::registry()
                .with(filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_target(false),
                )
                .with(flame_layer)
                .init();

            tracing::info!(
                stage = "tracing_init",
                flame_file = %flame_path.display(),
                "flamegraph tracing initialized"
            );
        } else {
            // Console-only mode
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(false)
                .init();
        }
    });
}

/// Flush the flamegraph by dropping the guard.
/// Call this at the end of a test to ensure the SVG is written.
pub fn flush_flame() {
    if let Some(guard_storage) = FLAME_GUARD.get() {
        if let Ok(mut guard) = guard_storage.lock() {
            // Drop the guard to flush the flamegraph to disk
            guard.take();
        }
    }
}

/// Get the output path for the flamegraph file.
fn flame_path(label: &str) -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join(format!("flame-{}.svg", label))
}

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
        // Note: opentelemetry-stdout 0.32 writes to stdout via println!
        // The OTel spans appear in test output (stdout).
        // AI-readable JSON events are written to file via the JSON layer below.
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
