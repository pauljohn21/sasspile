//! gen_spec_dataset.rs — Extract sass-spec HRX into a standalone JSON dataset.
//!
//! Produces a pure data file: no dependency on any Sass compiler.
//! Any implementation can use it as conformance reference.
//!
//! Usage:
//! ```sh
//! RUST_LOG=info rust-script scripts/gen_spec_dataset.rs --spec-root sass-spec/spec --output spec_dataset.json
//! ```
//! ```cargo
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! tracing = "0.1"
//! tracing-subscriber = { version = "0.3", features = ["env-filter"] }
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

// ─── HRX Parser (self-contained) ───────────────────────────────────────────

fn parse_hrx(content: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with("<===> ") {
            if let Some(name) = current_name.take() {
                files.push((name, current_content.trim_end_matches('\n').to_string()));
                current_content.clear();
            }
            current_name = Some(line.trim_start_matches("<===> ").trim().to_string());
        } else if line.starts_with("=======") || (line.starts_with("<===") && !line.starts_with("<===> ")) {
            if let Some(name) = current_name.take() {
                files.push((name, current_content.trim_end_matches('\n').to_string()));
                current_content.clear();
            }
        } else if current_name.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if let Some(name) = current_name {
        files.push((name, current_content.trim_end_matches('\n').to_string()));
    }
    files
}

fn find_hrx_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { results.extend(find_hrx_files(&path)); }
            else if path.extension().map(|e| e == "hrx").unwrap_or(false) { results.push(path); }
        }
    }
    results.sort();
    results
}

// ─── Dataset Types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct SpecFile { path: String, content: String }

#[derive(Debug, Serialize)]
struct SpecTestCase {
    id: String,
    domain: String,
    hrx_file: String,
    case_name: String,
    files: Vec<SpecFile>,
    entry: String,
    expected_output: Option<String>,
    expected_error: Option<String>,
    options: Option<String>,
    is_multi_file: bool,
}

#[derive(Debug, Serialize)]
struct SpecDomain { name: String, total_cases: usize, total_hrx: usize }

#[derive(Debug, Serialize)]
struct SpecDataset {
    version: String,
    total_cases: usize,
    total_hrx: usize,
    domains: Vec<SpecDomain>,
    test_cases: Vec<SpecTestCase>,
}

// ─── Extract ────────────────────────────────────────────────────────────────

fn extract_cases(hrx_path: &Path, domain: &str) -> Vec<SpecTestCase> {
    let content = match fs::read_to_string(hrx_path) { Ok(c) => c, Err(_) => return Vec::new() };
    let files = parse_hrx(&content);
    let hrx_name = hrx_path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or("unknown".to_string());

    let mut groups: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (path, content) in &files {
        let parent = match path.rfind('/') { Some(i) => path[..i].to_string(), None => String::new() };
        groups.entry(parent).or_default().push((path.clone(), content.clone()));
    }

    let mut cases = Vec::new();
    for (dir, gf) in &groups {
        if !gf.iter().any(|(p, _)| p.ends_with("input.scss")) { continue; }

        let entry = if dir.is_empty() { "input.scss".to_string() } else { format!("{}/input.scss", dir) };
        let expected_output = gf.iter().find(|(p, _)| p.ends_with("output.css")).map(|(_, c)| c.clone());
        let expected_error = gf.iter().find(|(p, _)| p.ends_with("/error") || *p == "error").map(|(_, c)| c.clone());
        let options = gf.iter().find(|(p, _)| p.ends_with("/options") || *p == "options").map(|(_, c)| c.clone());
        let is_multi_file = gf.len() > 2;
        let case_name = if dir.is_empty() { hrx_name.clone() } else { format!("{}_{}", hrx_name, dir.replace('/', "_")) };

        cases.push(SpecTestCase {
            id: format!("{}/{}", domain, case_name),
            domain: domain.to_string(),
            hrx_file: hrx_name.clone(),
            case_name,
            files: gf.iter().map(|(p, c)| SpecFile { path: p.clone(), content: c.clone() }).collect(),
            entry,
            expected_output,
            expected_error,
            options,
            is_multi_file,
        });
    }
    cases
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let span = tracing::info_span!("gen_dataset", stage = "gen_dataset");
    let _enter = span.enter();

    let args: Vec<String> = std::env::args().collect();
    let mut spec_root = String::from("sass-spec/spec");
    let mut output_path = String::from("spec_dataset.json");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--spec-root" => { i += 1; if i < args.len() { spec_root = args[i].clone(); } }
            "--output" | "-o" => { i += 1; if i < args.len() { output_path = args[i].clone(); } }
            _ => {}
        }
        i += 1;
    }

    let spec_root = PathBuf::from(&spec_root);
    if !spec_root.exists() {
        tracing::error!(path = %spec_root.display(), "spec root not found");
        std::process::exit(1);
    }

    tracing::info!(path = %spec_root.display(), "scanning for HRX files");

    let domains: &[(&str, &str)] = &[
        ("css/plain", "css_plain"), ("css", "css"), ("directives", "directives"),
        ("expressions", "expressions"), ("operators", "operators"), ("parser", "parser"),
        ("values", "values"), ("variables", "variables"), ("callable", "callable"),
        ("core_functions/color", "core_functions_color"),
        ("core_functions/list", "core_functions_list"),
        ("core_functions/map", "core_functions_map"),
        ("core_functions/math", "core_functions_math"),
        ("core_functions/meta", "core_functions_meta"),
        ("core_functions/string", "core_functions_string"),
        ("core_functions/selector", "core_functions_selector"),
        ("core_functions", "core_functions_misc"),
    ];

    let mut all_cases = Vec::new();
    let mut domain_stats = Vec::new();
    let mut total_hrx = 0usize;

    for (path, name) in domains {
        let dir = spec_root.join(path);
        if !dir.exists() {
            tracing::warn!(domain = %name, path = %path, "skipping (not found)");
            continue;
        }
        let hrx_files = find_hrx_files(&dir);
        total_hrx += hrx_files.len();
        let mut domain_cases = 0;
        for hrx in &hrx_files {
            let cases = extract_cases(hrx, name);
            domain_cases += cases.len();
            all_cases.extend(cases);
        }
        tracing::info!(
            domain = %name,
            hrx_count = hrx_files.len(),
            case_count = domain_cases,
            "domain scanned"
        );
        domain_stats.push(SpecDomain { name: name.to_string(), total_cases: domain_cases, total_hrx: hrx_files.len() });
    }

    let dataset = SpecDataset {
        version: "1.0".to_string(),
        total_cases: all_cases.len(),
        total_hrx,
        domains: domain_stats,
        test_cases: all_cases,
    };

    let json = serde_json::to_string_pretty(&dataset).unwrap_or_default();
    fs::write(&output_path, &json).unwrap_or_else(|e| {
        tracing::error!(error = %e, path = %output_path, "failed to write dataset");
        std::process::exit(1);
    });

    tracing::info!(
        output = %output_path,
        total_cases = dataset.total_cases,
        total_hrx = dataset.total_hrx,
        "dataset written"
    );
}
