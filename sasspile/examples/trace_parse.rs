//! Generate JSON trace logs for tracing-ai analysis.
//!
//! Usage:
//!   cargo run -p sasspile --example trace_parse -- input.scss [output.json]
//!
//! Analyze with tracing-ai:
//!   tracing-ai_analyze_traces --log-file output.json --slow-threshold-ms 10

use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use sasspile::{tokenize, parse};

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input.scss> [output.json]", args[0]);
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = args.get(2).map(|s| PathBuf::from(s)).unwrap_or_else(|| {
        let mut p = input_path.clone();
        p.set_extension("json");
        p
    });

    // Set up tracing-subscriber with JSON to a file
    let file = File::create(&output_path)?;
    let writer = Mutex::new(file);

    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_writer(writer)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                .unwrap_or_else(|_| "debug".into()),
        )
        .finish();

    let _guard = tracing::subscriber::set_default(subscriber);

    // Run parsing — all spans will go to the JSON file
    let source = std::fs::read_to_string(&input_path).unwrap();
    let name = input_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    tracing::info!(file = %name, source_len = source.len(), "Starting parse");
    let (tokens, lex_diags) = tokenize(&source);
    let (lex_e, lex_w, _) = lex_diags.counts();
    tracing::info!(tokens = tokens.len(), lex_errors = lex_e, lex_warnings = lex_w, "Tokenize complete");
    
    let (stylesheet, parse_diags) = parse(&source);
    let (p_e, p_w, _) = parse_diags.counts();
    tracing::info!(
        nodes = stylesheet.nodes.len(),
        parse_errors = p_e,
        parse_warnings = p_w,
        "Parse complete"
    );

    if p_e > 0 {
        for err in parse_diags.errors() {
            tracing::error!(error = %err.message, "Parse error");
        }
        tracing::warn!(total_errors = p_e, "File failed to parse cleanly");
    } else {
        tracing::info!("File parsed successfully");
    }

    io::stdout().flush()?;
    eprintln!("✓ JSON trace written to: {}", output_path.display());
    Ok(())
}
