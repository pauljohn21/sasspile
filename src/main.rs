use std::path::PathBuf;

use tracing::instrument;

fn main() {
    // Initialize tracing for CLI error reporting
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        tracing::error!("Usage: sasspile <file.scss>");
        std::process::exit(1);
    }

    let path = PathBuf::from(&args[1]);
    match compile_file_cli(&path) {
        Ok(css) => print!("{}", css),
        Err(e) => {
            tracing::error!(error = %e, file = %path.display(), "compilation failed");
            std::process::exit(1);
        }
    }
}

#[instrument(name = "cli_compile", skip_all, fields(stage = "compile", file = %path.display()))]
fn compile_file_cli(path: &PathBuf) -> Result<String, sasspile::SassError> {
    sasspile::compile_file(path)
}
