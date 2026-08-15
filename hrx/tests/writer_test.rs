use hrx::{parse, write, Archive};

#[test]
fn write_single_file() {
    let mut archive = Archive::new();
    archive.add_file("input.scss", "a { color: red }");
    let hrx = write(&archive);
    assert!(hrx.contains("<===> input.scss"));
    assert!(hrx.contains("a { color: red }"));
}

#[test]
fn write_multiple_files() {
    let mut archive = Archive::new();
    archive.add_file("input.scss", "a { color: red }");
    archive.add_file("output.css", "a {\n  color: red;\n}");
    let hrx = write(&archive);
    assert!(hrx.contains("<===> input.scss"));
    assert!(hrx.contains("<===> output.css"));
}

#[test]
fn roundtrip_basic() {
    let original = "<===> input.scss\na { color: red }\n";
    let archive = parse(original).unwrap();
    let written = write(&archive);
    let archive2 = parse(&written).unwrap();
    assert_eq!(archive.len(), archive2.len());
    
    let f1 = archive.get_file("input.scss").unwrap();
    let f2 = archive2.get_file("input.scss").unwrap();
    assert_eq!(f1.contents, f2.contents);
}

#[test]
fn roundtrip_multiple_files() {
    let original = "\
<===> input.scss
a { color: red }

<===> output.css
a {
  color: red;
}
";
    let archive = parse(original).unwrap();
    let written = write(&archive);
    let archive2 = parse(&written).unwrap();
    
    assert_eq!(archive.len(), archive2.len());
    
    let f1 = archive.get_file("input.scss").unwrap();
    let f2 = archive2.get_file("input.scss").unwrap();
    assert_eq!(f1.contents, f2.contents);
}

#[test]
fn write_file_helper() {
    let hrx = hrx::writer::write_file("test.txt", "hello world");
    assert!(hrx.contains("<===> test.txt"));
    assert!(hrx.contains("hello world"));
}

#[test]
fn write_with_newline_ensures_trailing_newline() {
    let mut archive = Archive::new();
    archive.add_file("file.txt", "contents");
    let hrx = hrx::writer::write_with_newline(&archive);
    assert!(hrx.ends_with('\n'));
}

#[test]
fn write_empty_archive() {
    let archive = Archive::new();
    let hrx = write(&archive);
    assert_eq!(hrx, "");
}
