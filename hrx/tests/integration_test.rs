#[test]
fn test_validate_real_sass_spec_file() {
    // This test validates our parser against a real sass-spec HRX file
    let test_data = "\
<===> input.scss
ul {
  margin-left: 1em;
  li {
    list-style-type: none;
  }
}

<===> output.css
ul {
  margin-left: 1em;
}
ul li {
  list-style-type: none;
}
";

    let archive = hrx::parse(test_data).unwrap();
    assert_eq!(archive.len(), 2);
    
    let input = archive.get_file("input.scss").unwrap();
    assert!(input.contents.contains("list-style-type"));
    
    let output = archive.get_file("output.css").unwrap();
    assert!(output.contents.contains("ul li"));
}

#[test]
fn test_complex_spec_structure() {
    // Tests a structure similar to what sass-spec uses with error files
    let test_data = "\
<===> error/undefined_var/input.scss
a {b: $undefined}

<===> error/undefined_var/error
Error: Undefined variable.
  ,
1 | a {b: $undefined}
  |       ^^^^^^^^^^
  '
  input.scss 1:7  root stylesheet

<===>
================================================================================
<===> error/type_mismatch/input.scss
a {b: \"string\" + 1}

<===> error/type_mismatch/error
Error: Undefined operation \"string + 1\".
";

    let archive = hrx::parse(test_data).unwrap();
    assert!(archive.len() >= 2);
}

#[test]
fn test_from_bytes() {
    let input = b"<===> file.txt\nhello world\n";
    let archive = hrx::parser::parse_bytes(input).unwrap();
    assert_eq!(archive.len(), 1);
    let file = archive.get_file("file.txt").unwrap();
    assert_eq!(file.contents, "hello world");
}

#[test]
fn test_archive_stats() {
    let mut archive = hrx::Archive::new();
    archive.add_file("a.scss", "line1\nline2");
    archive.add_file("b.css", "body { color: red }");
    
    assert_eq!(archive.len(), 2);
    assert_eq!(archive.files().len(), 2);
    assert!(!archive.is_empty());
}

#[test]
fn test_archive_add_dir() {
    use hrx::models::DirEntry;
    
    let mut archive = hrx::Archive::new();
    let mut dir = DirEntry::new("subdir");
    dir.children.push(hrx::models::file("input.scss", "a { color: red }"));
    archive.add_dir(dir);
    
    assert_eq!(archive.len(), 1);
    assert!(archive.entries()[0].is_dir());
}
