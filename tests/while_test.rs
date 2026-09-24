use sasspile::compile;

#[test]
fn while_basic_increment() {
    let input = r#"
$i: 1;
@while $i < 4 {
  .item-#{$i} { width: 10px * $i; }
  $i: $i + 1;
}
"#;
    let result = compile(input);
    assert!(result.contains(".item-1"), "should have .item-1: {result}");
    assert!(result.contains(".item-2"), "should have .item-2: {result}");
    assert!(result.contains(".item-3"), "should have .item-3: {result}");
    assert!(!result.contains(".item-4"), "should NOT have .item-4: {result}");
}

#[test]
fn while_basic_decrement() {
    let input = r#"
$i: 3;
@while $i > 0 {
  .col-#{$i} { width: #{$i}0%; }
  $i: $i - 1;
}
"#;
    let result = compile(input);
    assert!(result.contains(".col-3"), "got: {result}");
    assert!(result.contains(".col-2"), "got: {result}");
    assert!(result.contains(".col-1"), "got: {result}");
}

#[test]
fn while_false_cond_never_executes() {
    let input = r#"@while false { .should-not-exist { content: "leak"; } }"#;
    let result = compile(input);
    assert!(!result.contains(".should-not-exist"), "got: {result}");
}

#[test]
fn while_with_le_arithmetic() {
    let input = r#"
$i: 2;
@while $i <= 8 {
  .size-#{$i} { font-size: #{$i}px; }
  $i: $i + 1;
}
"#;
    let result = compile(input);
    assert!(result.contains(".size-2"), "got: {result}");
    assert!(result.contains(".size-8"), "got: {result}");
    assert!(!result.contains(".size-9"), "got: {result}");
}

#[test]
fn while_multiple_rules_per_iter() {
    let input = r#"
$step: 1;
@while $step <= 2 {
  .a-#{$step} { top: #{$step}px; }
  .b-#{$step} { bottom: #{$step}px; }
  $step: $step + 1;
}
"#;
    let result = compile(input);
    assert!(result.contains(".a-1"), "got: {result}");
    assert!(result.contains(".b-1"), "got: {result}");
    assert!(result.contains(".a-2"), "got: {result}");
    assert!(result.contains(".b-2"), "got: {result}");
}
