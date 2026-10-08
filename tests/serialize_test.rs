use rx_scss::serialize::{serialize, Options};
use rx_scss::types::CssStmt;

#[test]
fn test_serialize_simple_rule() {
    let stmts = vec![CssStmt::Rule {
        selector: "body".into(),
        inner: vec![
            CssStmt::Decl { property: "color".into(), value: "red".into() },
            CssStmt::Decl { property: "margin".into(), value: "0".into() },
        ],
    }];
    let result = serialize(&stmts, &Options::expanded());
    assert!(result.contains("body"));
    assert!(result.contains("color: red;"));
    assert!(result.contains("margin: 0;"));
}

#[test]
fn test_serialize_compressed() {
    let stmts = vec![CssStmt::Rule {
        selector: ".a".into(),
        inner: vec![
            CssStmt::Decl { property: "color".into(), value: "blue".into() },
        ],
    }];
    let result = serialize(&stmts, &Options::compressed());
    assert!(!result.contains('\n'));
    assert!(result.contains("color:blue;"));
}

#[test]
fn test_is_invisible_empty_rule() {
    let empty = CssStmt::Rule { selector: "x".into(), inner: vec![] };
    assert!(empty.is_invisible());

    let non_empty = CssStmt::Rule {
        selector: "y".into(),
        inner: vec![CssStmt::Decl { property: "z".into(), value: "1".into() }],
    };
    assert!(!non_empty.is_invisible());
}

#[test]
fn test_format_selectors_expanded() {
    // Test multi-selector formatting indirectly through public serialize API
    let stmts = vec![CssStmt::Rule {
        selector: ":root, [data-bs-theme=light]".into(),
        inner: vec![
            CssStmt::Decl { property: "--bs-color".into(), value: "blue".into() },
        ],
    }];
    let result = serialize(&stmts, &Options::expanded());
    // Multi-selectors should be split across lines with comma separator
    assert!(result.contains(":root"));
    assert!(result.contains("[data-bs-theme=light]"));
    assert!(result.contains(",\n"));
}
