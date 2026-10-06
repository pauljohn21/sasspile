use rx_scss::pipeline::{from_string, collect_stream};
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
fn collect_stream_basic() {
    use rxrust::prelude::*;
    let stream = Shared::from_iter(vec!["a".to_string(), "b".to_string(), "c".to_string()]).box_it();
    let result = collect_stream(stream);
    assert_eq!(result, "abc");
}

#[test]
fn collect_stream_empty() {
    use rxrust::prelude::*;
    let stream: rx_scss::types::OutputStream = Shared::from_iter(Vec::<String>::new()).box_it();
    let result = collect_stream(stream);
    assert_eq!(result, "");
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
