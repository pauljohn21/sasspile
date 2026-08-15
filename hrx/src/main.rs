//! HRX CLI tool - parse, inspect, and create HRX archives.
//!
//! # Usage
//!
//! ```bash
//! # Parse an HRX file and list its entries
//! hrx list <file.hrx>
//!
//! # Extract a specific file from an HRX archive
//! hrx extract <file.hrx> <path>
//!
//! # Create an HRX archive from files
//! hrx create <file.hrx> <input.scss> <output.css>
//!
//! # Validate an HRX file
//! hrx validate <file.hrx>
//!
//! # Pretty-print/rewrite an HRX file
//! hrx format <file.hrx>
//! ```

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

use hrx::Archive;

#[derive(Parser)]
#[command(name = "hrx", about = "HRX archive toolkit", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// List all entries in an HRX archive
    List {
        /// Path to the HRX file
        file: String,
    },

    /// Extract a file from an HRX archive and print its contents
    Extract {
        /// Path to the HRX file
        file: String,
        /// Path of the entry to extract (e.g., "input.scss")
        path: String,
    },

    /// Create an HRX archive from individual files
    Create {
        /// Output HRX file path
        output: String,
        /// Input files (will be named by their basename)
        files: Vec<String>,
    },

    /// Validate an HRX file can be parsed correctly
    Validate {
        /// Path to the HRX file
        file: String,
    },

    /// Pretty-print/format an HRX file
    Format {
        /// Path to the HRX file
        file: String,
    },

    /// Show statistics about an HRX archive
    Stats {
        /// Path to the HRX file
        file: String,
    },
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

    // Support JSON output for tracing-ai diagnostics
    if std::env::var("TRACING_AI_JSON").is_ok() {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .with_current_span(false)
            .with_span_list(true)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(false)
            .with_level(true)
            .init();
    }
}

fn main() {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    debug!(command = std::any::type_name::<Commands>(), "starting hrx CLI");

    let result = match cli.command {
        Commands::List { file } => cmd_list(&file),
        Commands::Extract { file, path } => cmd_extract(&file, &path),
        Commands::Create { output, files } => cmd_create(&output, &files),
        Commands::Validate { file } => cmd_validate(&file),
        Commands::Format { file } => cmd_format(&file),
        Commands::Stats { file } => cmd_stats(&file),
    };

    if let Err(e) = result {
        error!("command failed: {}", e);
        std::process::exit(1);
    }
}

fn cmd_list(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("listing entries in {}", file);
    let contents = fs::read_to_string(file)?;
    let archive = hrx::parse(&contents)?;

    println!("Archive: {}", file);
    println!("Entries:");
    print_entries(archive.entries(), 1);
    Ok(())
}

fn print_entries(entries: &[hrx::Entry], indent: usize) {
    let prefix = "  ".repeat(indent);
    for entry in entries {
        match entry {
            hrx::Entry::File(f) => {
                println!("{}📄 {} ({} bytes)", prefix, f.path, f.contents.len());
            }
            hrx::Entry::Dir(d) => {
                println!("{}/ {} ({} children)", prefix, d.path, d.children.len());
                print_entries(&d.children, indent + 1);
            }
        }
    }
}

fn cmd_extract(file: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("extracting '{}' from {}", path, file);
    let contents = fs::read_to_string(file)?;
    let archive = hrx::parse(&contents)?;

    match archive.get_file(path) {
        Some(entry) => {
            print!("{}", entry.contents);
            // Ensure trailing newline
            if !entry.contents.ends_with('\n') {
                println!();
            }
        }
        None => {
            warn!("file '{}' not found in archive", path);
            eprintln!("Error: path '{}' not found in archive", path);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn cmd_create(output: &str, files: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    info!(?files, "creating HRX archive {}", output);

    let mut archive = Archive::new();

    for path_str in files {
        let path = Path::new(path_str);
        let name = path
            .file_name()
            .ok_or_else(|| format!("invalid file path: {}", path_str))?
            .to_string_lossy();

        debug!("reading file: {}", path_str);
        let contents = fs::read_to_string(path_str)?;
        archive.add_file(name.as_ref(), &contents);
    }

    let hrx_output = hrx::write(&archive);
    fs::write(output, &hrx_output)?;

    info!(
        entries = archive.len(),
        bytes = hrx_output.len(),
        "wrote HRX archive to {}",
        output
    );
    Ok(())
}

fn cmd_validate(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("validating {}", file);
    let contents = fs::read_to_string(file)?;

    match hrx::parse(&contents) {
        Ok(archive) => {
            println!("✓ Valid HRX archive");
            println!("  Top-level entries: {}", archive.len());
            println!("  Total files: {}", archive.files().len());
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Invalid HRX archive: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_format(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("formatting {}", file);
    let contents = fs::read_to_string(file)?;
    let archive = hrx::parse(&contents)?;
    let formatted = hrx::write(&archive);
    println!("{}", formatted);
    Ok(())
}

fn cmd_stats(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("computing stats for {}", file);
    let contents = fs::read_to_string(file)?;
    let archive = hrx::parse(&contents)?;

    let total_files = count_files(archive.entries());
    let total_dirs = count_dirs(archive.entries());
    let total_bytes: usize = archive
        .entries()
        .iter()
        .map(entry_size)
        .sum();

    println!("Archive Statistics: {}", file);
    println!("─────────────────────────────");
    println!("  Top-level entries: {}", archive.len());
    println!("  Total files:       {}", total_files);
    println!("  Total directories: {}", total_dirs);
    println!("  Total content:     {} bytes", total_bytes);
    println!("  File size:         {} bytes", contents.len());

    Ok(())
}

fn count_files(entries: &[hrx::Entry]) -> usize {
    let mut count = 0;
    for entry in entries {
        match entry {
            hrx::Entry::File(_) => count += 1,
            hrx::Entry::Dir(d) => count += count_files(&d.children),
        }
    }
    count
}

fn count_dirs(entries: &[hrx::Entry]) -> usize {
    let mut count = 0;
    for entry in entries {
        if let hrx::Entry::Dir(d) = entry {
            count += 1 + count_dirs(&d.children);
        }
    }
    count
}

fn entry_size(entry: &hrx::Entry) -> usize {
    match entry {
        hrx::Entry::File(f) => f.contents.len(),
        hrx::Entry::Dir(d) => d.children.iter().map(entry_size).sum(),
    }
}
