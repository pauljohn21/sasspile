//! Debug test v5 - trace inside real mixin

use rx_scss::builder::CompileBuilder;

#[test]
fn debug_trace_mixin_internal() {
    // Add visible debug output inside the mixin
    let scss = r#"
@mixin generate-utility($utility, $infix: "", $is-rfs-media-query: false) {
  $values: map-get($utility, values);

  // DEBUG: check if values is map or list
  .debug-values-type { v: type-of($values); }

  @if type-of($values) == "string" or type-of(nth($values, 1)) != "list" {
    $values: zip($values, $values);
    .debug-zipped { v: type-of($values); }
  }

  @each $key, $value in $values {
    .debug-key-#{$key} { v: $value; }

    $properties: map-get($utility, property);

    @if type-of($properties) == "string" {
      $properties: append((), $properties);
    }

    $property-class: if(map-has-key($utility, class), map-get($utility, class), nth($properties, 1));
    $property-class: if($property-class == null, "", $property-class);
    $property-class-modifier: if($key, if($property-class == "" and $infix == "", "", "-") + $key, "");

    .debug-pc-#{$key} { v: $property-class + $infix + $property-class-modifier; }

    @if $value != null {
      .debug-notnull-#{$key} { v: $value; }
      .#{$property-class + $infix + $property-class-modifier} {
        display: $value;
      }
    }
  }
}

$util: (property: display, class: d, values: (inline: inline, block: block, none: none));
@include generate-utility($util);
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_trace_mixin.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_trace_mixin.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_trace_mixin_map_values() {
    // Test: when values is a map (key-value), does @each work correctly?
    let scss = r#"
@mixin test-mixin($utility) {
  $values: map-get($utility, values);
  .debug-type { v: type-of($values); }
  @each $key, $value in $values {
    .debug-#{$key} { v: $value; }
  }
}
$util: (values: (inline: inline, block: block, none: none));
@include test-mixin($util);
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_trace_map.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_trace_map.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_trace_mixin_string_values() {
    // Test: when values is a list (space-separated), does zip work?
    let scss = r#"
@mixin test-mixin($utility) {
  $values: map-get($utility, values);
  .debug-type-before { v: type-of($values); }
  @if type-of($values) == "string" or type-of(nth($values, 1)) != "list" {
    $values: zip($values, $values);
    .debug-type-after { v: type-of($values); }
  }
  @each $key, $value in $values {
    .debug-#{$key} { v: $value; }
  }
}
$util: (values: inline block none);
@include test-mixin($util);
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_trace_string.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_trace_string.txt", format!("ERROR: {}", e)).ok();
        }
    }
}
