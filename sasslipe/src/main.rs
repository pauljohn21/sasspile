//! sasslipe CLI — command-line SCSS compiler.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use sasslipe::Compiler;

/// Command-line arguments.
#[derive(Parser)]
#[command(name = "sasslipe", about = "Pure Rust async SCSS compiler", version)]
struct Cli {
    /// Input SCSS file (stdin if not provided).
    input: Option<PathBuf>,

    /// Output CSS file (stdout if not provided).
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Watch mode — recompile on file changes.
    #[arg(short, long)]
    watch: bool,

    /// Enable verbose logging (-v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

fn init_logging(verbose: u8) {
    let filter = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_level(true)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    info!(version = sasslipe::VERSION, "sasslipe CLI starting");

    let compiler = Compiler::new();

    match cli.input {
        Some(path) => {
            info!(path = %path.display(), "compiling file");
            let css = compiler.compile_file(&path.to_string_lossy()).await?;

            if let Some(out) = cli.output {
                tokio::fs::write(&out, &css).await?;
                info!(path = %out.display(), "output written");
            } else {
                warn!("no output file specified, printing to stdout");
                println!("{css}");
            }
        }
        None => {
            warn!("reading from stdin not yet implemented");
        }
    }

    Ok(())
}
