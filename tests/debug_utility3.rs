//! Debug test v3 - isolate compound interpolation in selector

use rx_scss::builder::CompileBuilder;

#[test]
fn debug_selector_interp_concat() {
    // Test: selector with string concat interpolation
    let scss = r#"
$property-class: "d";
$infix: "";
$key: "inline";
$property-class-modifier: "-" + $key;
.#{$property-class + $infix + $property-class-modifier} {
  display: inline;
}
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_sel_interp.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_sel_interp.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_if_with_string_compare() {
    // Test: if($property-class == "" and $infix == "", "", "-")
    let scss = r#"
$property-class: "d";
$infix: "";
$key: "inline";
$modifier: if($property-class == "" and $infix == "", "", "-") + $key;
.test { v: $modifier; }
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_if_concat.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_if_concat.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_if_null_check() {
    // Test: if($property-class == null, "", $property-class)
    let scss = r#"
$class: null;
$result: if($class == null, "", $class);
.test1 { v: $result; }
$class2: "d";
$result2: if($class2 == null, "", $class2);
.test2 { v: $result2; }
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_if_null.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_if_null.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_full_mixin_minimal() {
    // Minimal version of the failing part
    let scss = r#"
@mixin gen-test($utility, $infix: "") {
  $values: map-get($utility, values);
  @each $key, $value in $values {
    $properties: map-get($utility, property);
    $property-class: if(map-has-key($utility, class), map-get($utility, class), nth($properties, 1));
    $property-class: if($property-class == null, "", $property-class);
    $property-class-modifier: if($key, if($property-class == "" and $infix == "", "", "-") + $key, "");
    .#{$property-class + $infix + $property-class-modifier} {
      display: $value;
    }
  }
}
$util: (property: display, class: d, values: (inline: inline, block: block));
@include gen-test($util);
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            let has_d = css.contains(".d-");
            std::fs::write("/tmp/debug_full_minimal.txt", format!(
                "has_d={} bytes={}\n\n{}", has_d, css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_full_minimal.txt", format!("ERROR: {}", e)).ok();
        }
    }
}

#[test]
fn debug_selector_interp_issue() {
    // What does #{"d" + "" + "-inline"} produce?
    let scss = r#"
$a: "d";
$b: "";
$c: "-inline";
.test { v: #{$a + $b + $c}; }
"#;
    let result = CompileBuilder::new().compile_string(scss);
    match result {
        Ok(css) => {
            std::fs::write("/tmp/debug_sel_concat.txt", format!(
                "bytes={}\n\n{}", css.len(), css
            )).ok();
        }
        Err(e) => {
            std::fs::write("/tmp/debug_sel_concat.txt", format!("ERROR: {}", e)).ok();
        }
    }
}
