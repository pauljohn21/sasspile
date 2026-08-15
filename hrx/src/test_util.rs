//! Test utilities for tracing initialization.
//!
//! These helpers enable JSON tracing output compatible with tracing-ai diagnostics.

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize tracing for test runs.
///
/// When `TRACING_AI_JSON` env var is set, outputs JSON format compatible with tracing-ai.
/// Otherwise outputs pretty format for local debugging.
pub fn init_test_tracing() {
    INIT.call_once(|| {
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"));

        if std::env::var("TRACING_AI_JSON").is_ok() {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(env_filter)
                .with_current_span(false)
                .with_span_list(true)
                .with_target(true)
                .init();
        } else {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_level(true)
                .init();
        }
    });
}

/// Initialize tracing with pretty output for local debugging.
pub fn init_test_tracing_pretty() {
    INIT.call_once(|| {
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"));

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_level(true)
            .init();
    });
}
