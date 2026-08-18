//! Tests for `ModuleCache` — module evaluation result caching.
//!
//! Verifies that:
//! - A module used by multiple files is only evaluated once.
//! - CSS output from a module is emitted only on the first `@use`.
//! - `@use with` on an already-loaded module produces an error.

use std::path::PathBuf;

struct TempScss {
    dir: PathBuf,
}

impl TempScss {
    fn new(test_name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "sasspile_module_cache_{}_{}_{}",
            std::process::id(),
            test_name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
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

/// Test: a module used by two files should only emit CSS once.
#[test]
fn test_module_css_emitted_once() {
    let tmp = TempScss::new("css_once");

    tmp.write("_shared.scss", ".shared { color: red; }\n");
    tmp.write(
        "_a.scss",
        "@use \"shared\";\n.a { background: white; }\n",
    );
    tmp.write(
        "_b.scss",
        "@use \"shared\";\n.b { border: 1px; }\n",
    );
    tmp.write(
        "main.scss",
        "@use \"a\";\n@use \"b\";\n",
    );

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    let shared_count = css.matches(".shared").count();
    assert_eq!(
        shared_count, 1,
        "expected .shared to appear exactly once, got {} times in: {}",
        shared_count, css
    );
}

/// Test: variables defined in a module are consistent across multiple @use sites.
#[test]
fn test_module_vars_consistent() {
    let tmp = TempScss::new("vars_consistent");

    tmp.write("_config.scss", "$primary: #ff0000;\n$radius: 4px;\n");
    tmp.write(
        "_a.scss",
        "@use \"config\";\n.a { color: config.$primary; }\n",
    );
    tmp.write(
        "_b.scss",
        "@use \"config\";\n.b { border-radius: config.$radius; }\n",
    );
    tmp.write(
        "main.scss",
        "@use \"a\";\n@use \"b\";\n",
    );

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(css.contains("#f00") || css.contains("red"), "expected red/#f00 in output, got: {}", css);
    assert!(css.contains("4px"), "expected '4px' in output, got: {}", css);
}

/// Test: module with side-effect (counter) — should only increment once.
#[test]
fn test_module_side_effect_once() {
    let tmp = TempScss::new("side_effect");

    // Module that defines a counter variable
    tmp.write("_counter.scss", "$count: 42;\n.counter { value: $count; }\n");
    tmp.write(
        "_first.scss",
        "@use \"counter\";\n.first { val: counter.$count; }\n",
    );
    tmp.write(
        "_second.scss",
        "@use \"counter\";\n.second { val: counter.$count; }\n",
    );
    tmp.write(
        "main.scss",
        "@use \"first\";\n@use \"second\";\n",
    );

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    // .counter should only appear once (from the first @use)
    let counter_count = css.matches(".counter").count();
    assert_eq!(
        counter_count, 1,
        "expected .counter once, got {} in: {}",
        counter_count, css
    );
}

/// Test: `@use with` on an already-loaded module should error.
#[test]
fn test_use_with_on_cached_errors() {
    let tmp = TempScss::new("with_cached_error");

    tmp.write("_mod.scss", "$val: red !default;\n.mod { color: $val; }\n");
    tmp.write(
        "_a.scss",
        "@use \"mod\" with ($val: blue);\n",
    );
    tmp.write(
        "_b.scss",
        "@use \"mod\" with ($val: green);\n",
    );
    tmp.write(
        "main.scss",
        "@use \"a\";\n@use \"b\";\n",
    );

    let result = sasspile::compile_file(tmp.main_path());
    assert!(
        result.is_err(),
        "expected compilation to fail when configuring an already-loaded module, but got: {:?}",
        result.ok()
    );
}
