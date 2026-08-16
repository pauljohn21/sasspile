//! Basic SCSS compilation — simplest usage of the sasspile API.
//!
//! Usage:
//!   cargo run -p sasspile --example basic_compile

use sasspile::Compiler;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for progress output.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let compiler = Compiler::new();

    // Example 1: variables and nesting.
    let scss = r#"
$primary: #3498db;
$padding: 16px;

.container {
    padding: $padding;
    background: lighten($primary, 20%);

    h1 {
        color: $primary;
        font-size: 2em;
    }
}
"#;

    tracing::info!("=== Example 1: Variables & Nesting ===");
    match compiler.compile(scss).await {
        Ok(css) => tracing::info!(css_len = css.len(), "compiled successfully"),
        Err(e) => tracing::error!(error = %e, "compilation failed"),
    }

    // Example 2: built-in functions.
    let scss2 = r#"
$base-color: #ff0000;

.box {
    color: $base-color;
    background: complement($base-color);
    border-color: darken($base-color, 15%);
}
"#;

    tracing::info!("=== Example 2: Built-in Color Functions ===");
    match compiler.compile(scss2).await {
        Ok(css) => tracing::info!(css_len = css.len(), "compiled successfully"),
        Err(e) => tracing::error!(error = %e, "compilation failed"),
    }

    // Example 3: list and map operations.
    let scss3 = r#"
$colors: red, green, blue;
$theme: (
    primary: #3498db,
    secondary: #2ecc71
);

.first-color {
    color: nth($colors, 1);
}

.map-get {
    color: map.get($theme, primary);
}
"#;

    tracing::info!("=== Example 3: Lists & Maps ===");
    match compiler.compile(scss3).await {
        Ok(css) => tracing::info!(css_len = css.len(), "compiled successfully"),
        Err(e) => tracing::error!(error = %e, "compilation failed"),
    }

    tracing::info!("=== All examples complete ===");
}
