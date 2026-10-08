//! Debug test to isolate utility generation failure

use rx_scss::builder::CompileBuilder;

#[test]
fn debug_generate_utility_real() {
    // Test the actual generate-utility mixin from Bootstrap
    let scss = std::fs::read_to_string("bootstrap/scss/mixins/_utilities.scss").unwrap();
    let result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compile_string(&scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_gen_util_raw_css.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_gen_util_raw_css.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_generate_utility_call() {
    // Test calling generate-utility with a simple utility
    let scss = "@import \"functions\";\n@import \"variables\";\n@import \"maps\";\n@import \"mixins\";\n\n@include generate-utility((property: display, class: d, values: (inline: inline, block: block, none: none)));\n";
    let result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_gen_util_call_css.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_gen_util_call_css.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_generate_utility_string_values() {
    // Test with string values (like Bootstrap's "display" utility)
    let scss = "@import \"functions\";\n@import \"variables\";\n@import \"maps\";\n@import \"mixins\";\n\n$utility: (property: display, class: d, values: inline block none);\n@include generate-utility($utility);\n";
    let result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_gen_util_str_css.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_gen_util_str_css.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_typeof_string_or_list() {
    // The condition: type-of($values) == "string" or type-of(nth($values, 1)) != "list"
    let scss = "@import \"functions\";\n@import \"variables\";\n@import \"maps\";\n@import \"mixins\";\n\n$values: inline block none;\n@debug type-of($values);\n@debug type-of(nth($values, 1));\n@debug type-of($values) == \"string\";\n@debug type-of(nth($values, 1)) != \"list\";\n.test { v: type-of($values); }\n.test2 { v: type-of(nth($values, 1)); }\n";
    let result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_typeof_cond_css.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_typeof_cond_css.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_zip_values() {
    // Test zip($values, $values) which is used when values is a string/list
    let scss = "@import \"functions\";\n@import \"variables\";\n@import \"maps\";\n@import \"mixins\";\n\n$values: inline block none;\n$zipped: zip($values, $values);\n.test { len: length($zipped); }\n@each $key, $value in $zipped {\n  .z-#{$key} { v: $value; }\n}\n";
    let result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_zip_css.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_zip_css.txt", format!("ERROR: {}", e)).ok();
        }
    }
}
