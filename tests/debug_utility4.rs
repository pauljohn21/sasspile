//! Debug test v4 - reality check with real generate-utility

use rx_scss::builder::CompileBuilder;

#[test]
fn debug_real_mixin_no_rfs() {
    // Real mixin but disable rfs by removing the rfs block
    let scss = r#"
@mixin generate-utility($utility, $infix: "", $is-rfs-media-query: false) {
  $values: map-get($utility, values);

  @if type-of($values) == "string" or type-of(nth($values, 1)) != "list" {
    $values: zip($values, $values);
  }

  @each $key, $value in $values {
    $properties: map-get($utility, property);

    @if type-of($properties) == "string" {
      $properties: append((), $properties);
    }

    $property-class: if(map-has-key($utility, class), map-get($utility, class), nth($properties, 1));
    $property-class: if($property-class == null, "", $property-class);

    $css-variable-name: if(map-has-key($utility, css-variable-name), map-get($utility, css-variable-name), map-get($utility, class));

    $state: if(map-has-key($utility, state), map-get($utility, state), ());

    $infix: if($property-class == "" and str-slice($infix, 1, 1) == "-", str-slice($infix, 2), $infix);

    $property-class-modifier: if($key, if($property-class == "" and $infix == "", "", "-") + $key, "");

    $is-css-var: map-get($utility, css-var);
    $is-local-vars: map-get($utility, local-vars);
    $is-rtl: map-get($utility, rtl);

    @if $value != null {
      @if $is-css-var {
        .#{$property-class + $infix + $property-class-modifier} {
          --#{$prefix}#{$css-variable-name}: #{$value};
        }
      } @else {
        .#{$property-class + $infix + $property-class-modifier} {
          @each $property in $properties {
            #{$property}: $value !important;
          }
        }
      }
    }
  }
}

$prefix: "bs-";
$enable-important-utilities: true;
$util: (property: display, class: d, values: (inline: inline, block: block, none: none));
@include generate-utility($util);
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_real_norfs.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_real_norfs.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_str_slice_issue() {
    // Test: str-slice function used in real mixin
    let scss = r#"
$infix: "-sm";
$first: str-slice($infix, 1, 1);
.test { v: $first; }
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_str_slice.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_str_slice.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_bs_prefix_interp() {
    // Test: --#{$prefix}#{$css-variable-name} in property name
    let scss = r#"
$prefix: "bs-";
$name: "display";
.test { --#{$prefix}#{$name}: inline; }
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_bs_prefix.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_bs_prefix.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_bootstrap_import_utilities_only() {
    // What happens with just @import "utilities"?
    let scss = "@import \"functions\";\n@import \"variables\";\n@import \"maps\";\n@import \"mixins\";\n@import \"utilities\";\n";
    let result = CompileBuilder::new().include_path("bootstrap/scss/").compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            let has_m = css.contains(".m-");
            let has_p = css.contains(".p-");
            std::fs::write("/tmp/debug_import_only.txt", format!(
                "has_d={} has_m={} has_p={} bytes={}\n\n{}",
                has_d, has_m, has_p, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_import_only.txt", format!("ERROR: {}", e)).ok();
        }
    }
}
