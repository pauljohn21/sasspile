//! Tests for `@use "module" with ($var: value)` configuration injection.
//!
//! These tests verify that configuration variables are correctly injected
//! into the module's evaluation environment, overriding `!default` values.

use std::path::PathBuf;

/// Create a temp directory with SCSS files for @use tests.
struct TempScss {
    dir: PathBuf,
}

impl TempScss {
    fn new(test_name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "sasspile_use_config_{}_{}_{}",
            std::process::id(),
            test_name,
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Self { dir }
    }

    fn write(&self, name: &str, content: &str) {
        let path = self.dir.join(name);
        std::fs::write(&path, content).unwrap();
    }

    fn main_path(&self) -> PathBuf {
        self.dir.join("main.scss")
    }
}

impl Drop for TempScss {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Test: single config variable injection.
#[test]
fn test_use_with_single_config() {
    let tmp = TempScss::new("single");

    tmp.write("_module.scss", "$primary: #ff0000 !default;\n.color { color: $primary; }\n");
    tmp.write("main.scss", "@use \"module\" with ($primary: #336699);\n");

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(
        css.contains("#369"),
        "expected #369 (short form of #336699) in output, got: {}", css
    );
}

/// Test: multiple config variables injection.
#[test]
fn test_use_with_multiple_config() {
    let tmp = TempScss::new("multiple");

    tmp.write("_module.scss", "$color: blue !default;\n$size: 10px !default;\n.box { color: $color; font-size: $size; }\n");
    tmp.write("main.scss", "@use \"module\" with ($color: red, $size: 16px);\n");

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(css.contains("red"), "expected 'red' in output, got: {}", css);
    assert!(css.contains("16px"), "expected '16px' in output, got: {}", css);
}

/// Test: config variable referencing an existing variable.
#[test]
fn test_use_with_var_reference() {
    let tmp = TempScss::new("var_ref");

    tmp.write("_module.scss", "$primary: black !default;\n.text { color: $primary; }\n");
    tmp.write("main.scss", "$theme-color: #00ff00;\n@use \"module\" with ($primary: $theme-color);\n");

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(
        css.contains("#0f0") || css.contains("#00ff00"),
        "expected #0f0 or #00ff00 in output, got: {}", css
    );
}

/// Test: unconfigured !default variable keeps its default value.
#[test]
fn test_use_default_preserved() {
    let tmp = TempScss::new("default_preserved");

    tmp.write("_module.scss", "$primary: red !default;\n$gap: 10px !default;\n.layout { color: $primary; gap: $gap; }\n");
    tmp.write("main.scss", "@use \"module\" with ($primary: blue);\n");

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(css.contains("blue"), "expected 'blue' in output, got: {}", css);
    assert!(css.contains("10px"), "expected default '10px' in output, got: {}", css);
}
