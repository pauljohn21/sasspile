use hrx::parser::parse;
use hrx::models::Entry;

#[test]
fn parse_single_file() {
    let input = "<===> input.scss\na { color: red }\n";
    let archive = parse(input).unwrap();
    assert_eq!(archive.len(), 1);
    match &archive.entries()[0] {
        Entry::File(f) => {
            assert_eq!(f.path, "input.scss");
            assert_eq!(f.contents, "a { color: red }");
        }
        _ => panic!("expected file entry"),
    }
}

#[test]
fn parse_multiple_files() {
    let input = "\
<===> input.scss
a { color: red }

<===> output.css
a {
  color: red;
}
";
    let archive = parse(input).unwrap();
    assert_eq!(archive.len(), 2);

    let file1 = archive.get_file("input.scss").unwrap();
    assert_eq!(file1.contents, "a { color: red }");

    let file2 = archive.get_file("output.css").unwrap();
    assert!(file2.contents.contains("color: red"));
}

#[test]
fn parse_nested_paths() {
    let input = "\
<===> dir1/input.scss
a { color: blue }

<===> dir1/_partial.scss
@import 'partial';
";
    let archive = parse(input).unwrap();
    assert_eq!(archive.len(), 2);
}

#[test]
fn parse_with_directory_boundary() {
    let input = "\
<===> unbracketed/input.scss
a {b: is-bracketed(foo bar)}

<===> unbracketed/output.css
a {b: false}

<===>
================================================================================
<===> bracketed/input.scss
a {b: is-bracketed([foo bar])}

<===> bracketed/output.css
a {b: true}
";
    let archive = parse(input).unwrap();
    assert!(archive.len() > 0);
}

#[test]
fn parse_empty_contents() {
    let input = "<===> empty.txt\n";
    let archive = parse(input).unwrap();
    assert_eq!(archive.len(), 1);
    let file = archive.get_file("empty.txt").unwrap();
    assert_eq!(file.contents, "");
}

#[test]
fn parse_multiline_contents() {
    let input = "\
<===> input.sass
a
  color: red
  font-size: 14px

  .nested
    display: none
";
    let archive = parse(input).unwrap();
    let file = archive.get_file("input.sass").unwrap();
    assert!(file.contents.contains("color: red"));
    assert!(file.contents.contains(".nested"));
}

#[test]
fn get_file_returns_none_for_missing() {
    let input = "<===> file.txt\ncontent\n";
    let archive = parse(input).unwrap();
    assert!(archive.get_file("nonexistent.txt").is_none());
}

#[test]
fn parse_real_sass_spec_format() {
    let input = "\
<===> global/input.scss
@use \"sass:math\" as *;
a {b: compatible(1px, 1in)}

<===> global/output.css
a {
  b: true;
}
";
    let archive = parse(input).unwrap();
    let input_file = archive.get_file("global/input.scss").unwrap();
    assert!(input_file.contents.contains("@use \"sass:math\""));

    let output_file = archive.get_file("global/output.css").unwrap();
    assert!(output_file.contents.contains("b: true"));
}
