//! HRX test archive loader — reads sass-spec HRX file trees.
//!
//! Provides utilities to load sass-spec test cases from HRX archives,
//! extracting input.scss and expected_output.css pairs.

use std::path::Path;

use tracing::{debug, info, warn};

/// A loaded sass-spec test case.
#[derive(Debug, Clone)]
pub struct SpecCase {
    /// Test name (derived from archive path).
    pub name: String,
    /// Input SCSS text.
    pub input: String,
    /// Expected CSS output.
    pub expected: String,
    /// Optional output style override.
    pub style: Option<String>,
}

/// Load a single HRX file as a spec test case.
pub fn load_hrx_file(path: impl AsRef<Path>) -> Result<SpecCase, String> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| format!("read failed: {e}"))?;

    let archive = hrx::parse(&content).map_err(|e| format!("hrx parse: {e}"))?;

    extract_spec_case(&archive, path.to_string_lossy().as_ref())
}

/// Extract a spec case from a parsed HRX archive.
fn extract_spec_case(archive: &hrx::Archive, source_path: &str) -> Result<SpecCase, String> {
    // Determine test name from input file.
    let name = derive_test_name(source_path);

    // Look for input.scss.
    let input = find_input(archive)?;
    let expected = find_expected(archive)?;
    let style = find_style(archive);

    info!(%name, "loaded spec case");
    Ok(SpecCase {
        name,
        input,
        expected,
        style,
    })
}

/// Derive a test name from the file path.
fn derive_test_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Find the input SCSS in the archive.
fn find_input(archive: &hrx::Archive) -> Result<String, String> {
    // Try common input names.
    for name in &["input.scss", "input.sass"] {
        if let Some(entry) = archive.get_file(name) {
            return Ok(entry.contents.clone());
        }
    }
    // Try nested input.
    for entry in archive.entries() {
        if let hrx::Entry::File(f) = entry {
            if f.path.contains("input.scss") || f.path.contains("input.sass") {
                return Ok(f.contents.clone());
            }
        }
    }
    Err("no input.scss found in archive".to_string())
}

/// Find the expected output CSS.
fn find_expected(archive: &hrx::Archive) -> Result<String, String> {
    for name in &[
        "expected_output.css",
        "output.css",
        "expected.css",
        "expected_output",
    ] {
        if let Some(entry) = archive.get_file(name) {
            return Ok(entry.contents.clone());
        }
    }
    for entry in archive.entries() {
        if let hrx::Entry::File(f) = entry {
            if f.path.contains("expected") && f.path.ends_with(".css") {
                return Ok(f.contents.clone());
            }
        }
    }
    Err("no expected_output.css found in archive".to_string())
}

/// Find output style override in options.yml or similar.
fn find_style(archive: &hrx::Archive) -> Option<String> {
    for name in &["options.yml", "options.yaml", "options"] {
        if let Some(entry) = archive.get_file(name) {
            // Parse simple YAML `style: expanded`.
            for line in entry.contents.lines() {
                if let Some((key, val)) = line.split_once(':') {
                    if key.trim() == "style" {
                        return Some(val.trim().to_string());
                    }
                }
            }
        }
    }
    None
}

/// Load all HRX files in a directory.
pub fn load_hrx_dir(dir: impl AsRef<Path>) -> Vec<Result<SpecCase, String>> {
    let dir = dir.as_ref();
    let mut results = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            warn!(error = %e, "failed to read spec directory");
            return vec![];
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("hrx") {
            debug!(path = %path.display(), "loading HRX");
            results.push(load_hrx_file(&path));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_valid_hrx() {
        // Test loading from a manually-created HRX.
        let hrx_content = "<===> input.scss\n.foo { color: red; }\n\n<===> expected_output.css\n.foo {\n  color: red;\n}\n";
        let archive = hrx::parse(hrx_content).expect("should parse");
        let case = extract_spec_case(&archive, "test.hrx").expect("should extract");

        assert_eq!(case.name, "test");
        assert!(case.input.contains(".foo"));
        assert!(case.expected.contains("color: red"));
    }

    #[test]
    fn load_missing_input_fails() {
        let hrx_content = "<===> expected_output.css\n.foo { color: red; }\n";
        let archive = hrx::parse(hrx_content).expect("should parse");
        let result = extract_spec_case(&archive, "test.hrx");
        assert!(result.is_err());
    }

    #[test]
    fn derive_test_name_from_path() {
        assert_eq!(derive_test_name("/foo/bar/test_case.hrx"), "test_case");
        assert_eq!(derive_test_name("simple.hrx"), "simple");
    }
}
