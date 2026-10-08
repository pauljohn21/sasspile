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
        .partition(is_at_root);

    // Expanded 模式：展平嵌套规则以匹配标准 CSS 输出
    match opts.style {
        OutputStyle::Expanded => {
            let flat = flatten_stmts(&normal_stmts, "");
            serialize_stmts(&flat, opts, &mut output, 0);
            let flat_atroot = flatten_stmts(&atroot_stmts, "");
            serialize_stmts(&flat_atroot, opts, &mut output, 0);
        }
        _ => {
            serialize_stmts(&normal_stmts, opts, &mut output, 0);
            serialize_stmts(&atroot_stmts, opts, &mut output, 0);
        }
    }
    output
}

/// 展平嵌套 CSS 规则为扁平结构。
/// parent 参数用于组合选择器（".parent" + " " + ".child" → ".parent .child"）。
fn flatten_stmts(stmts: &[CssStmt], parent: &str) -> Vec<CssStmt> {
    let mut result = Vec::new();
    for stmt in stmts {
        match stmt {
            CssStmt::Rule { selector, inner } => {
                let selector = strip_at_root_marker(selector);
                // 组合父选择器
                let combined = combine_selectors(parent, &selector);
                // 分离声明和嵌套规则
                let (decls, nested): (Vec<_>, Vec<_>) = inner.iter().cloned().partition(|s| matches!(s, CssStmt::Decl { .. }));
                // 输出当前规则的声明
                if !decls.is_empty() {
                    result.push(CssStmt::Rule {
                        selector: combined.clone(),
                        inner: decls,
                    });
                }
                // 递归展平嵌套规则
                let nested_flat = flatten_stmts(&nested, &combined);
                result.extend(nested_flat);
            }
            CssStmt::Media { query, inner } => {
                // @media 内部的规则也需要展平。
                // 非顶层上下文中，需要将 Decl 包裹进父选择器 Rule
                // 否则 Decl 在 flatten 时被丢弃（parent 非空）
                let normalized = if parent.is_empty() {
                    inner.clone()
                } else {
                    inner.iter().cloned().map(|s| match s {
                        CssStmt::Decl { .. } => CssStmt::Rule {
                            selector: parent.to_string(),
                            inner: vec![s],
                        },
                        _ => s,
                    }).collect()
                };
                let flat_inner = flatten_stmts(&normalized, parent);
                result.push(CssStmt::Media {
                    query: query.clone(),
                    inner: flat_inner,
                });
            }
            CssStmt::Supports { query, inner } => {
                let normalized = if parent.is_empty() {
                    inner.clone()
                } else {
                    inner.iter().cloned().map(|s| match s {
                        CssStmt::Decl { .. } => CssStmt::Rule {
                            selector: parent.to_string(),
                            inner: vec![s],
                        },
                        _ => s,
                    }).collect()
                };
                let flat_inner = flatten_stmts(&normalized, parent);
                result.push(CssStmt::Supports {
                    query: query.clone(),
                    inner: flat_inner,
                });
            }
            // 顶层声明（无父选择器包裹）— 直接传递
            // 非顶层上下文中的 Decl 进不到这里（已由 Media/Supports 的包裹逻辑处理）
            CssStmt::Decl { .. } => {
                if parent.is_empty() {
                    result.push(stmt.clone());
                }
            }
            CssStmt::Comment(_) | CssStmt::Charset => {
                result.push(stmt.clone());
            }
        }
    }
    result
}

/// 组合父选择器与子选择器。
fn combine_selectors(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        return child.to_string();
    }
    if let Some(stripped) = child.strip_prefix('&') {
        // &.class → parentclass（移除 & 并直接拼接）
        format!("{}{}", parent, stripped)
    } else if child.starts_with(':') || child.starts_with('[') {
        // :pseudo 或 [attr] → 直接拼接
        format!("{}{}", parent, child)
    } else {
        // 默认：后代选择器
        format!("{} {}", parent, child)
    }
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
