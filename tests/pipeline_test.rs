use rx_scss::pipeline::from_string;
use rx_scss::serialize::Options;

#[test]
fn from_string_simple_rule() {
    let result = from_string("body { color: red; }", &Options::default());
    assert!(result.is_ok());
    let css = result.unwrap();
    assert!(css.contains("body"));
    assert!(css.contains("color: red;"));
}

#[test]
fn from_string_empty_input() {
    let result = from_string("", &Options::default());
    assert!(result.is_ok());
}

#[test]
fn from_string_compressed_output() {
    let opts = Options {
        style: rx_scss::types::OutputStyle::Compressed,
        suppress_charset: true,
    };
    let result = from_string(".a { color: blue; }", &opts).unwrap();
    assert!(result.contains("color:blue;"));
    assert!(!result.contains('\n'));
}

#[test]
fn from_string_variable_and_interpolation() {
    let result = from_string("$color: green; .x { color: $color; }", &Options::default());
    assert!(result.is_ok());
    let css = result.unwrap();
    assert!(css.contains("color: green;"));
}

#[test]
fn from_string_with_options_charset() {
    let opts = Options {
        style: rx_scss::types::OutputStyle::Expanded,
        suppress_charset: false,
    };
    let result = from_string("div{}", &opts).unwrap();
    assert!(result.contains("@charset"), "should include charset by default");
}

#[test]
fn from_string_suppress_charset() {
    let opts = Options {
        style: rx_scss::types::OutputStyle::Expanded,
        suppress_charset: true,
    };
    let result = from_string("div{}", &opts).unwrap();
    assert!(!result.contains("@charset"), "should suppress charset when requested");
}

#[test]
fn compound_selector_attribute() {
    // 属性选择器 `[type="checkbox"]` 应 compound 组合（无空格）
    let result = from_string(
        ".btn-check { &[type=\"checkbox\"] { margin: 0; } }",
        &Options::default(),
    )
    .unwrap();
    assert!(
        result.contains(".btn-check[type=checkbox]"),
        "attribute selector should be compound (no space). Got: {}",
        result
    );
}

#[test]
fn compound_selector_pseudo_class() {
    // 伪类 `:focus` 应 compound 组合（无空格）
    let result = from_string(
        ".btn { &:focus { outline: none; } }",
        &Options::default(),
    )
    .unwrap();
    assert!(
        result.contains(".btn:focus"),
        "pseudo-class should be compound (no space). Got: {}",
        result
    );
}

#[test]
fn compound_selector_pseudo_element() {
    // 伪元素 `::after` 应 compound 组合（无空格）
    let result = from_string(
        ".card { &::after { content: \"\"; } }",
        &Options::default(),
    )
    .unwrap();
    assert!(
        result.contains(".card::after"),
        "pseudo-element should be compound (no space). Got: {}",
        result
    );
}

#[test]
fn descendant_selector_class() {
    // 普通类选择器嵌套应产生空格分隔的 descendant 组合子
    let result = from_string(
        ".card { .title { font-weight: bold; } }",
        &Options::default(),
    )
    .unwrap();
    assert!(
        result.contains(".card .title"),
        "class selector should produce descendant combinator (space). Got: {}",
        result
    );
}

#[test]
fn compound_selector_multiple_children() {
    // 逗号分隔的子选择器中 `[attr]` 和 `:pseudo` 各自独立判断
    let result = from_string(
        ".parent { &[type=\"x\"], &:hover { color: red; } }",
        &Options::default(),
    )
    .unwrap();
    assert!(
        result.contains(".parent[type=x]"),
        "attribute child should be compound. Got: {}",
        result
    );
    assert!(
        result.contains(".parent:hover"),
        "pseudo-class child should be compound. Got: {}",
        result
    );
}
