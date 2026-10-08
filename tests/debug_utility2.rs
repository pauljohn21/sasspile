//! Debug test v2 - isolate generate-utility mixin failure

use rx_scss::builder::CompileBuilder;

#[test]
fn debug_mixin_step1() {
    // Just the first few lines of generate-utility
    let scss = r#"
@mixin gen-test($utility, $infix: "") {
  $values: map-get($utility, values);
  @each $key, $value in $values {
    .d-#{$key} { display: $value; }
  }
}
$util: (values: (inline: inline, block: block, none: none));
@include gen-test($util);
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_mixin_step1.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_mixin_step1.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_mixin_step2() {
    // Add the type-of check
    let scss = r#"
@mixin gen-test($utility, $infix: "") {
  $values: map-get($utility, values);
  @if type-of($values) == "string" or type-of(nth($values, 1)) != "list" {
    $values: zip($values, $values);
  }
  @each $key, $value in $values {
    .d-#{$key} { display: $value; }
  }
}
$util: (values: (inline: inline, block: block, none: none));
@include gen-test($util);
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_mixin_step2.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_mixin_step2.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_mixin_step3() {
    // Add $properties and more checks
    let scss = r#"
@mixin gen-test($utility, $infix: "") {
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
    $property-class-modifier: if($key, if($property-class == "" and $infix == "", "", "-") + $key, "");
    @if $value != null {
      .#{$property-class + $infix + $property-class-modifier} {
        #{$properties}: $value;
      }
    }
  }
}
$util: (property: display, class: d, values: (inline: inline, block: block, none: none));
@include gen-test($util);
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_mixin_step3.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_mixin_step3.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_mixin_step4() {
    // Test with the real Bootstrap generate-utility mixin content, copied exactly
    let mixin_content = std::fs::read_to_string("bootstrap/scss/mixins/_utilities.scss").unwrap();
    let test = format!("{}\n$util: (property: display, class: d, values: (inline: inline, block: block, none: none));\n@include generate-utility($util);\n", mixin_content);
    let result = CompileBuilder::new().include_path("bootstrap/scss/").compile_string(&test);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_mixin_step4.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_mixin_step4.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_Compound_interpolation() {
    // Test: #{$property-class + $infix + $property-class-modifier}
    let scss = r#"
$property-class: "d";
$infix: "";
$key: "inline";
$property-class-modifier: "-" + $key;
.test { v: #{$property-class + $infix + $property-class-modifier}; }
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_compound_interp.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_compound_interp.txt", format!("ERROR: {}", e)).ok();
        }
    }
}
