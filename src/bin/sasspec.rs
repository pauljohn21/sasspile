use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use futures::stream::StreamExt;
use tracing::info_span;

use sasspile::{compile, compile_with_files};

struct SpecTest {
    name: String,
    input: String,
    expected_output: String,
    files: HashMap<String, String>,
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

static FLAT_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn parse_hrx_files(content: &str) -> Vec<(String, String)> {
    // First, collect raw files (path, content) preserving order
    let mut raw_files: Vec<(String, String)> = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_content: Vec<String> = Vec::new();

    content.lines().for_each(|line| {
        if let Some(path) = line.strip_prefix("<===>") {
            if let Some(p) = current_path.take() {
                raw_files.push((p, current_content.join("\n")));
                current_content.clear();
            }
            current_path = Some(path.trim().to_string());
        } else if line.trim() == "<==>" {
            if let Some(p) = current_path.take() {
                raw_files.push((p, current_content.join("\n")));
                current_content.clear();
            }
        } else if !line.starts_with("================") && !line.starts_with("---") {
            current_content.push(line.to_string());
        }
    });
    if let Some(p) = current_path.take() {
        raw_files.push((p, current_content.join("\n")));
    }

    // Now group into test sections: each input.scss file starts a new test
    // Test name = directory containing input.scss; other files share the prefix
    // Group by test: each input.scss starts a new test; discard pre-input metadata
    let mut result: Vec<(String, String)> = Vec::new();
    let mut current_test_name: Option<String> = None;
    let mut current_test_files: Vec<(String, String)> = Vec::new();

    for (path, content) in raw_files {
        let is_input = path.ends_with("/input.scss")
            || path.ends_with("/input.sass")
            || path == "input.scss"
            || path == "input.sass";

        if is_input {
            // Discard any pre-input metadata (README, options) — they belong to no test
            if current_test_name.is_none() {
                current_test_files.clear();
            } else {
                // Flush previous test
                result.push((format!("__test_sep__{}", current_test_name.unwrap()), String::new()));
                result.extend(current_test_files.drain(..));
            }
            // Derive test name: "a/b/input.scss" -> "a/b"
            let raw_name = path
                .trim_end_matches("/input.scss")
                .trim_end_matches("/input.sass")
                .trim_end_matches("input.scss")
                .trim_end_matches("input.sass")
                .trim_start_matches('/')
                .to_string();
            let name = if raw_name.is_empty() {
                let id = FLAT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                format!("__flat_{}", id)
            } else {
                raw_name
            };
            current_test_name = Some(name);
        }
        current_test_files.push((path, content));
    }
    // Flush last test
    if let Some(name) = current_test_name.take() {
        result.push((format!("__test_sep__{}", name), String::new()));
        result.extend(current_test_files.drain(..));
    }

    result
}

fn group_into_test_cases(files: Vec<(String, String)>) -> Vec<SpecTest> {
    let mut cur_name: Option<String> = None;
    let mut cur_files: HashMap<String, String> = HashMap::new();
    let mut tests: Vec<SpecTest> = Vec::new();

    for (file_path, content) in files {
        if file_path.starts_with("__test_sep__") {
            // Flush previous test group
            if let Some(name) = cur_name.take() {
                if let Some(test) = build_test(name, &cur_files) {
                    tests.push(test);
                }
                cur_files.clear();
            }
            cur_name = Some(file_path.trim_start_matches("__test_sep__").to_string());
            continue;
        }
        // Strip test prefix (e.g., "test_name/subdir/file.scss" -> "subdir/file.scss")
        if let Some(ref name) = cur_name {
            let rel = file_path
                .strip_prefix(name)
                .and_then(|s| s.strip_prefix('/'))
                .unwrap_or(&file_path);
            cur_files.insert(rel.to_string(), content);
        }
    }
    // Flush last test group
    if let Some(name) = cur_name.take() {
        if let Some(test) = build_test(name, &cur_files) {
            tests.push(test);
        }
        cur_files.clear();
    }

    tests
}

fn build_test(name: String, files: &HashMap<String, String>) -> Option<SpecTest> {
    let input = files
        .get("input.scss")
        .or_else(|| files.get("input.sass"))?
        .clone();
    let output = files.get("output.css")?.clone();
    // Collect auxiliary files (everything except input/output)
    let aux: HashMap<String, String> = files
        .iter()
        .filter(|(k, _)| k.as_str() != "input.scss" && k.as_str() != "input.sass" && k.as_str() != "output.css")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    Some(SpecTest {
        name,
        input,
        expected_output: output,
        files: aux,
    })
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
                let actual = if test.files.is_empty() {
                    compile(&test.input)
                } else {
                    compile_with_files(&test.input, &test.files)
                };
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

    // Categorize failures by first path segment
    let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    failed.iter().for_each(|r| {
        let cat = r.name.split('/').next().unwrap_or("").to_string();
        *categories.entry(cat).or_default() += 1;
    });
    let mut cats: Vec<_> = categories.iter().collect();
    cats.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    cats.iter().take(20).for_each(|(cat, count)| {
        tracing::warn!(category = %cat, count = count, "failure category");
    });

    // Write full failure report to /tmp/sasspec_failures.tsv for diagnostic
    let mut failure_output = String::new();
    for r in &failed {
        failure_output.push_str(&format!(
            "{}\t{}\t{}\n",
            r.name,
            r.expected.replace('\n', "\\n").replace('\t', "\\t"),
            r.actual.replace('\n', "\\n").replace('\t', "\\t")
        ));
    }
    let _ = std::fs::write("/tmp/sasspec_failures.tsv", &failure_output);

    failed.iter().take(50).for_each(|r| {
        tracing::warn!(
            test = %r.name,
            expected = %r.expected,
            actual = %r.actual,
            "failed test"
        );
    });
}

