//! Debug test v6 - test real Bootstrap utilities compilation

use rx_scss::builder::CompileBuilder;

#[test]
fn debug_full_utilities_with_api() {
    // Compile the exact chain that Bootstrap uses
    let scss = "@import \"functions\";\n@import \"variables\";\n@import \"variables-dark\";\n@import \"maps\";\n@import \"mixins\";\n@import \"utilities\";\n";
    let result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            let has_text = css.contains(".text-");
            let has_bg = css.contains(".bg-");
            let has_border = css.contains(".border-");
            let has_gap = css.contains(".gap-");
            let has_margin = css.contains(".m-");
            let has_padding = css.contains(".p-");
            std::fs::write("/tmp/debug_full_util_chain.txt", format!(
                "has_d={} has_text={} has_bg={} has_border={} has_gap={} has_margin={} has_padding={} bytes={}\n\n{}",
                has_d, has_text, has_border, has_bg, has_gap, has_margin, has_padding, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_full_util_chain.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_api_only_with_imports() {
    // Just the api.scss part (no utilities/api import)
    let scss = "@import \"functions\";\n@import \"variables\";\n@import \"variables-dark\";\n@import \"maps\";\n@import \"mixins\";\n@import \"utilities\";\n\n@each $breakpoint in map-keys($grid-breakpoints) {\n  @include media-breakpoint-up($breakpoint) {\n    $infix: breakpoint-infix($breakpoint, $grid-breakpoints);\n    @each $key, $utility in $utilities {\n      @if type-of($utility) == \"map\" and (map-get($utility, responsive) or $infix == \"\") {\n        @include generate-utility($utility, $infix);\n      }\n    }\n  }\n}\n";
    let result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_api_direct.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_api_direct.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_simple_gen_call_after_imports() {
    // After imports, try calling generate-utility directly
    let scss = "@import \"functions\";\n@import \"variables\";\n@import \"variables-dark\";\n@import \"maps\";\n@import \"mixins\";\n@import \"utilities\";\n\n@include generate-utility((property: display, class: d, values: (inline: inline, block: block, none: none)));\n";
    let result = CompileBuilder::new()
        .include_path("bootstrap/scss/")
        .compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_gen_after_imports.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_gen_after_imports.txt", format!("ERROR: {}", e)).ok();
        }
    }
}
