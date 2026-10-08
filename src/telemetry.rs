//! Tracing initialization for rx-scss.
//!
//! Provides initialization functions for tracing with proper filtering.

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize tracing with OpenTelemetry-compatible stdout exporter.
///
/// Sets up:
/// - EnvFilter for log level control via RUST_LOG env var
/// - Thread IDs and target information in output
/// - JSON format for production, pretty format for development
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init();
}

/// Initialize tracing for tests with stderr output.
///
/// This variant writes to stderr so that test output can be captured
/// with `--nocapture`.
pub fn init_test_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_test_writer()
        .try_init();
}
