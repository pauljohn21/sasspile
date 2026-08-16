//! Concurrent compilation — batch compile multiple SCSS files in parallel.
//!
//! Demonstrates the `Pipeline::compile_batch` API which spawns Tokio tasks
//! for each input and collects results as they complete.
//!
//! Usage:
//!   cargo run -p sasspile --example concurrent_compile

use sasspile::css::OutputStyle;
use sasspile::{Pipeline, PipelineInput};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("=== Concurrent Compilation Example ===\n");

    // Define multiple inputs to compile in parallel.
    let inputs = vec![
        PipelineInput {
            path: "buttons.scss".to_string(),
            source: r#"
$btn-color: #0d6efd;
.btn {
    color: white;
    background: $btn-color;
    &:hover { background: darken($btn-color, 10%); }
}
"#
            .to_string(),
        },
        PipelineInput {
            path: "grid.scss".to_string(),
            source: r#"
$columns: 12;
$gap: 1rem;

@for $i from 1 through $columns {
    .col-#{$i} {
        width: percentage($i / $columns);
        padding: 0 $gap / 2;
    }
}
"#
            .to_string(),
        },
        PipelineInput {
            path: "theme.scss".to_string(),
            source: r#"
$palette: (
    primary: #0d6efd,
    success: #198754,
    warning: #ffc107,
    danger: #dc3545
);

@each $name, $color in $palette {
    .bg-#{$name} { background: $color; }
    .text-#{$name} { color: $color; }
}
"#
            .to_string(),
        },
    ];

    tracing::info!(files = inputs.len(), "starting concurrent compilation");

    let pipeline = Pipeline::new();
    let results = pipeline.compile_batch(inputs, OutputStyle::Expanded).await;

    tracing::info!(completed = results.len(), "batch complete");

    let mut success = 0;
    let mut failed = 0;

    for result in results {
        match result {
            Ok(output) => {
                success += 1;
                tracing::info!(
                    path = %output.path,
                    css_bytes = output.css.len(),
                    "compiled OK"
                );
            }
            Err(e) => {
                failed += 1;
                tracing::error!(error = %e, "compilation failed");
            }
        }
    }

    tracing::info!(
        success = success,
        failed = failed,
        "=== Concurrent compilation complete ==="
    );
}
