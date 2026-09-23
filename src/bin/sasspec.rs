use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use futures::stream::StreamExt;
use tracing::info_span;

use sasspile::compile;

struct SpecTest {
    name: String,
    input: String,
    expected_output: String,
}

#[derive(Debug, Clone)]
struct TestResult {
    name: String,
    passed: bool,
    expected: String,
    actual: String,
}

const BLACKLIST_DIRS: &[&str] = &[
    "non_conformant",
    "libsass",
    "libsass-closed-issues",
    "libsass-todo-issues",
    "libsass-todo-tests",
];

const FOCUS_DIR: &str = "directives";

fn find_hrx_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        entries.flatten().for_each(|entry| {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Skip blacklisted directories
                if BLACKLIST_DIRS.contains(&dir_name) {
                    return;
                }
                // Only focus on directives directory (skip others at top level)
                if dir.file_name().and_then(|n| n.to_str()) == Some("spec")
                    && dir_name != FOCUS_DIR
                {
                    return;
                }
                files.extend(find_hrx_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("hrx") {
                files.push(path);
            }
        });
    }
    files
}

fn parse_hrx_files(content: &str) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_content: Vec<String> = Vec::new();

    content.lines().for_each(|line| {
        if let Some(path) = line.strip_prefix("<===>") {
            if let Some(path) = current_path.take() {
                files.push((path, current_content.join("\n")));
                current_content.clear();
            }
            current_path = Some(path.trim().to_string());
        } else if line.trim() == "<==>" {
            if let Some(path) = current_path.take() {
                files.push((path, current_content.join("\n")));
                current_content.clear();
            }
        } else if !line.starts_with("================") {
            current_content.push(line.to_string());
        }
    });

    if let Some(path) = current_path {
        files.push((path, current_content.join("\n")));
    }

    files
}

fn group_into_test_cases(files: Vec<(String, String)>) -> Vec<SpecTest> {
    let mut groups: HashMap<String, HashMap<String, String>> = HashMap::new();

    files.into_iter().for_each(|(file_path, content)| {
        let parts: Vec<&str> = file_path.split('/').collect();
        if parts.len() >= 2 {
            let test_name = parts[..parts.len() - 1].join("/");
            let file_name = parts[parts.len() - 1].to_string();
            groups
                .entry(test_name)
                .or_default()
                .insert(file_name, content);
        }
    });

    groups
        .into_iter()
        .filter_map(|(name, files)| {
            let input = files
                .get("input.scss")
                .or_else(|| files.get("input.sass"))?
                .clone();
            let output = files.get("output.css")?.clone();
            Some(SpecTest {
                name,
                input,
                expected_output: output,
            })
        })
        .collect()
}

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let spec_dir = Path::new("sass-spec/spec");
    let hrx_files = find_hrx_files(spec_dir);

    let test_cases: Vec<SpecTest> = hrx_files
        .iter()
        .flat_map(|file| {
            let content = fs::read_to_string(file).unwrap_or_default();
            let files = parse_hrx_files(&content);
            group_into_test_cases(files)
        })
        .collect();

    let total = test_cases.len();

    // Spawn all compilations as futures, then stream results
    let futures: Vec<_> = test_cases
        .into_iter()
        .map(|test| {
            let span = info_span!("sasspec", test = %test.name);
            async move {
                let _enter = span.enter();
                let actual = compile(&test.input);
                let actual_trimmed = actual.trim().to_string();
                let expected_trimmed = test.expected_output.trim().to_string();
                TestResult {
                    name: test.name,
                    passed: actual_trimmed == expected_trimmed,
                    expected: expected_trimmed,
                    actual: actual_trimmed,
                }
            }
        })
        .collect();

    // Use futures::stream to run concurrently, then collect reactively
    let results: Vec<TestResult> = futures::stream::iter(futures)
        .buffer_unordered(total)
        .collect()
        .await;

    let passed = results.iter().filter(|r| r.passed).count();
    let failed: Vec<&TestResult> = results.iter().filter(|r| !r.passed).collect();

    tracing::info!(
        passed = passed,
        total = total,
        pass_rate = if total > 0 { passed as f64 / total as f64 * 100.0 } else { 0.0 },
        "sass-spec results"
    );

    failed.iter().take(20).for_each(|r| {
        tracing::warn!(
            test = %r.name,
            expected = %r.expected,
            actual = %r.actual,
            "failed test"
        );
    });
}
