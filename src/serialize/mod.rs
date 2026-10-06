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

    serialize_stmts(stmts, opts, &mut output, 0);
    output
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
                    let indent = "  ".repeat(indent_level + 1);
                    out.push_str(&format!("{}{}: {};\n", indent, property, value));
                }
                OutputStyle::Nested => {
                    let indent = "  ".repeat(indent_level + 1);
                    out.push_str(&format!("{}{}: {};\n", indent, property, value));
                }
                OutputStyle::Compressed => {
                    out.push_str(&format!("{}:{};", property, value));
                }
            }
        }
        CssStmt::Rule { selector, inner } => {
            match opts.style {
                OutputStyle::Expanded => {
                    let indent = "  ".repeat(indent_level);
                    out.push_str(&format!("{}{{\n", selector));
                    serialize_stmts(inner, opts, out, indent_level + 1);
                    out.push_str(&format!("{}}}\n", indent));
                }
                OutputStyle::Nested => {
                    let indent = "  ".repeat(indent_level);
                    out.push_str(&format!("{}{{\n", selector));
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
}
