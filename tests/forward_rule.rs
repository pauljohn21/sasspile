//! Tests for `@forward "url"` directive.
//!
//! Verifies that:
//! - `@forward` re-exports variables, functions, and mixins from a module.
//! - `show` filter limits forwarded members.
//! - `hide` filter excludes specific members.
//! - `@forward` does NOT produce CSS output itself.

use std::path::PathBuf;

struct TempScss {
    dir: PathBuf,
}

impl TempScss {
    fn new(test_name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "sasspile_forward_{}_{}_{}",
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

/// Test: `@forward` re-exports variables so downstream @use can access them.
#[test]
fn test_forward_variables() {
    let tmp = TempScss::new("forward_vars");

    tmp.write("_base.scss", "$primary: #ff0000;\n$secondary: #00ff00;\n");
    // _forwarder.scss forwards _base.scss's members
    tmp.write("_forwarder.scss", "@forward \"base\";\n");
    // main.scss uses forwarder and accesses forwarded variables
    tmp.write(
        "main.scss",
        "@use \"forwarder\" as fwd;\n.a { color: fwd.$primary; background: fwd.$secondary; }\n",
    );

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(
        css.contains("#f00") || css.contains("red"),
        "expected red/#f00 in output, got: {}",
        css
    );
    assert!(
        css.contains("#0f0") || css.contains("#00ff00"),
        "expected green/#0f0 in output, got: {}",
        css
    );
}

/// Test: `@forward` does NOT produce CSS output from the forwarded module.
#[test]
fn test_forward_no_css_output() {
    let tmp = TempScss::new("forward_no_css");

    // _base.scss has CSS rules
    tmp.write("_base.scss", ".base-rule { color: red; }\n");
    // _forwarder.scss forwards _base.scss
    tmp.write("_forwarder.scss", "@forward \"base\";\n.forwarder-only { display: block; }\n");
    // main.scss uses forwarder
    tmp.write("main.scss", "@use \"forwarder\";\n");

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    // .base-rule should NOT appear because @forward doesn't emit CSS
    assert!(
        !css.contains(".base-rule"),
        "expected .base-rule to NOT appear (forward doesn't emit CSS), got: {}",
        css
    );
    // .forwarder-only SHOULD appear because it's defined in the forwarder itself
    assert!(
        css.contains(".forwarder-only"),
        "expected .forwarder-only to appear, got: {}",
        css
    );
}

/// Test: `@forward` re-exports mixins.
#[test]
fn test_forward_mixins() {
    let tmp = TempScss::new("forward_mixins");

    tmp.write("_base.scss", "@mixin bold-text { font-weight: bold; }\n");
    // _forwarder.scss forwards _base.scss's members and also defines its own
    tmp.write("_forwarder.scss", "@forward \"base\";\n");
    // main.scss uses forwarder with no namespace so mixins are accessible directly
    tmp.write(
        "main.scss",
        "@use \"forwarder\";\n.text { @include bold-text; }\n",
    );

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(
        css.contains("bold"),
        "expected 'bold' in output, got: {}",
        css
    );
}

/// Test: `@forward` re-exports functions.
#[test]
fn test_forward_functions() {
    let tmp = TempScss::new("forward_functions");

    tmp.write("_base.scss", "@function double($n) { @return $n * 2; }\n");
    tmp.write("_forwarder.scss", "@forward \"base\";\n");
    tmp.write(
        "main.scss",
        "@use \"forwarder\" as fwd;\n.val { width: fwd.double(5px); }\n",
    );

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(
        css.contains("10px"),
        "expected '10px' in output, got: {}",
        css
    );
}

/// Test: `show` filter limits forwarded members.
#[test]
fn test_forward_show_filter() {
    let tmp = TempScss::new("forward_show");

    tmp.write(
        "_base.scss",
        "$visible: red;\n$hidden: blue;\n",
    );
    // Only forward $visible, not $hidden
    tmp.write(
        "_forwarder.scss",
        "@forward \"base\" show $visible;\n",
    );
    tmp.write(
        "main.scss",
        "@use \"forwarder\" as fwd;\n.a { color: fwd.$visible; }\n",
    );

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(
        css.contains("#f00") || css.contains("red"),
        "expected red/#f00 in output, got: {}",
        css
    );
}

/// Test: `hide` filter excludes specific members.
#[test]
fn test_forward_hide_filter() {
    let tmp = TempScss::new("forward_hide");

    tmp.write(
        "_base.scss",
        "$keep: red;\n$drop: blue;\n",
    );
    // Forward everything except $drop
    tmp.write(
        "_forwarder.scss",
        "@forward \"base\" hide $drop;\n",
    );
    tmp.write(
        "main.scss",
        "@use \"forwarder\" as fwd;\n.a { color: fwd.$keep; }\n",
    );

    let css = sasspile::compile_file(tmp.main_path()).expect("compilation should succeed");
    assert!(
        css.contains("#f00") || css.contains("red"),
        "expected red/#f00 in output, got: {}",
        css
    );
}
