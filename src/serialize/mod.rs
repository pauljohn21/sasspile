use crate::types::{CssStmt, OutputStyle};

#[derive(Debug, Clone)]
pub struct Options {
    pub style: OutputStyle,
    pub suppress_charset: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            style: OutputStyle::Expanded,
            suppress_charset: false,
        }
    }
}

impl Options {
    pub fn expanded() -> Self {
        Self {
            style: OutputStyle::Expanded,
            suppress_charset: false,
        }
    }

    pub fn compressed() -> Self {
        Self {
            style: OutputStyle::Compressed,
            suppress_charset: false,
        }
    }

    pub fn nested() -> Self {
        Self {
            style: OutputStyle::Nested,
            suppress_charset: false,
        }
    }

    pub fn with_style(mut self, style: OutputStyle) -> Self {
        self.style = style;
        self
    }

    pub fn suppress_charset(mut self, suppress: bool) -> Self {
        self.suppress_charset = suppress;
        self
    }
}

const CHARSET: &str = "@charset \"UTF-8\";\n";

pub fn serialize(stmts: &[CssStmt], opts: &Options) -> String {
    let mut output = String::new();

    if !opts.suppress_charset {
        match opts.style {
            OutputStyle::Expanded => output.push_str(CHARSET),
            OutputStyle::Nested => output.push_str(CHARSET),
            OutputStyle::Compressed => {} // no charset comment in compressed
        }
    }

    // Separate @at-root rules for hoisting
    let (atroot_stmts, normal_stmts): (Vec<_>, Vec<_>) = stmts.iter()
        .cloned()
        .partition(|s| is_at_root(s));

    serialize_stmts(&normal_stmts, opts, &mut output, 0);
    serialize_stmts(&atroot_stmts, opts, &mut output, 0);
    output
}

/// Check if a CssStmt is an @at-root marker.
fn is_at_root(stmt: &CssStmt) -> bool {
    match stmt {
        CssStmt::Rule { selector, .. } => selector.starts_with("/*@at-root*/ "),
        _ => false,
    }
}

/// Strip the /*@at-root*/ marker prefix from a selector.
fn strip_at_root_marker(selector: &str) -> String {
    selector.strip_prefix("/*@at-root*/ ").unwrap_or(selector).to_string()
}

/// Format comma-separated selectors for Expanded/Nested style.
/// Always prepends the base_indent (Bootstrap format: indented selectors inside @media).
/// Multi-selectors are separated by ",\n" with each on its own indented line.
fn format_selectors_expanded(selector: &str, indent_level: usize) -> String {
    let base_indent = "  ".repeat(indent_level);
    let parts: Vec<&str> = selector.split(',').collect();
    let formatted: Vec<String> = parts.iter()
        .map(|s| format!("{}{}", base_indent, s.trim()))
        .collect();
    if parts.len() <= 1 {
        return formatted[0].clone();
    }
    format!("{},\n{}", formatted[0], formatted[1..].join(",\n"))
}

fn serialize_stmts(stmts: &[CssStmt], opts: &Options, out: &mut String, indent_level: usize) {
    for stmt in stmts {
        serialize_stmt(stmt, opts, out, indent_level);
    }
}

fn serialize_stmt(stmt: &CssStmt, opts: &Options, out: &mut String, indent_level: usize) {
    if stmt.is_invisible() {
        return;
    }

    match stmt {
        CssStmt::Decl { property, value } => {
            match opts.style {
                OutputStyle::Expanded => {
                    let indent = "  ".repeat(indent_level);
                    out.push_str(&format!("{}{}: {};\n", indent, property, value));
                }
                OutputStyle::Nested => {
                    let indent = "  ".repeat(indent_level);
                    out.push_str(&format!("{}{}: {};\n", indent, property, value));
                }
                OutputStyle::Compressed => {
                    out.push_str(&format!("{}:{};", property, value));
                }
            }
        }
        CssStmt::Rule { selector, inner } => {
            let selector = strip_at_root_marker(selector);
            match opts.style {
                OutputStyle::Expanded => {
                    let indent = "  ".repeat(indent_level);
                    let formatted = format_selectors_expanded(&selector, indent_level);
                    out.push_str(&format!("{} {{\n", formatted));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push_str(&format!("{}}}\n", indent));
                }
                OutputStyle::Nested => {
                    let indent = "  ".repeat(indent_level);
                    let formatted = format_selectors_expanded(&selector, indent_level);
                    out.push_str(&format!("{} {{\n", formatted));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push_str(&format!("{}}}\n", indent));
                }
                OutputStyle::Compressed => {
                    out.push_str(&format!("{}{{", selector));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push('}');
                }
            }
        }
        CssStmt::Media { query, inner } => {
            match opts.style {
                OutputStyle::Expanded => {
                    out.push_str(&format!("@media {} {{\n", query));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push_str("}\n");
                }
                OutputStyle::Nested => {
                    out.push_str(&format!("@media {} {{\n", query));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push_str("}\n");
                }
                OutputStyle::Compressed => {
                    out.push_str(&format!("@media{}{{", query));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push('}');
                }
            }
        }
        CssStmt::Supports { query, inner } => {
            match opts.style {
                OutputStyle::Expanded => {
                    out.push_str(&format!("@supports {} {{\n", query));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push_str("}\n");
                }
                OutputStyle::Nested => {
                    out.push_str(&format!("@supports {} {{\n", query));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push_str("}\n");
                }
                OutputStyle::Compressed => {
                    out.push_str(&format!("@supports{}{{", query));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push('}');
                }
            }
        }
        CssStmt::Comment(text) => {
            match opts.style {
                OutputStyle::Expanded | OutputStyle::Nested => {
                    if !text.starts_with('!') || !matches!(opts.style, OutputStyle::Compressed) {
                        out.push_str(&format!("/*{}*/\n", text));
                    }
                }
                OutputStyle::Compressed => {
                    if text.starts_with('!') {
                        out.push_str(&format!("/*{}*/", text));
                    }
                }
            }
        }
        CssStmt::Charset => {
            out.push_str(CHARSET);
        }
    }
}

#[cfg(test)]
mod serialize_tests {
    use super::*;

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
        assert_eq!(format_selectors_expanded(":root", 0), ":root");
        let multi = format_selectors_expanded(":root, [data-bs-theme=light]", 0);
        assert!(multi.contains(":root,"));
        assert!(multi.contains("[data-bs-theme=light]"));
    }
}
