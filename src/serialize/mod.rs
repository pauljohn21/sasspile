use crate::types::{CssStmt, OutputStyle, CssStream, OutputStream};
use rxrust::prelude::*;

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
/// 注意：evaluator 已经完成了选择器组合（combine_selectors），
/// 所以 CssStmt::Rule 中的 selector 已经是最终形式，无需再次组合。
/// parent 参数仅用于在非顶层 Media/Supports 上下文中包裹 Decl 节点。
fn flatten_stmts(stmts: &[CssStmt], parent: &str) -> Vec<CssStmt> {
    let mut result = Vec::new();
    for stmt in stmts {
        match stmt {
            CssStmt::Rule { selector, inner } => {
                let selector = strip_at_root_marker(selector);
                // Strip placeholder selectors (%name) — they are removed from output when extended
                let selector = strip_placeholders(&selector);
                // 分离声明和嵌套规则
                let (decls, nested): (Vec<_>, Vec<_>) = inner.iter().cloned().partition(|s| matches!(s, CssStmt::Decl { .. }));
                // 输出当前规则的声明（选择器已是最终形式，直接使用）
                if !decls.is_empty() && !selector.is_empty() {
                    result.push(CssStmt::Rule {
                        selector: selector.clone(),
                        inner: decls,
                    });
                }
                // 递归展平嵌套规则（selector 已是最终组合结果，传空 parent 避免重复组合）
                let nested_flat = flatten_stmts(&nested, "");
                result.extend(nested_flat);
            }
            CssStmt::Media { query, inner } => {
                // @media 内部：evaluator 已组合选择器，直接展平子规则
                let flat_inner = flatten_stmts(inner, "");
                result.push(CssStmt::Media {
                    query: query.clone(),
                    inner: flat_inner,
                });
            }
            CssStmt::Supports { query, inner } => {
                let flat_inner = flatten_stmts(inner, "");
                result.push(CssStmt::Supports {
                    query: query.clone(),
                    inner: flat_inner,
                });
            }
            CssStmt::Decl { .. } => {
                // 顶层声明直接输出；非顶层 Decl 已在 evaluator 层被 Rule 包裹
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

/// Strip placeholder selectors (%name) from a comma-separated selector string.
/// Placeholders should not appear in CSS output — they are only used as @extend targets.
fn strip_placeholders(selector: &str) -> String {
    let entries: Vec<&str> = selector.split(',')
        .map(str::trim)
        .filter(|e| !e.starts_with('%'))
        .collect();
    entries.join(", ")
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
            // Strip placeholder selectors (%name) — they are removed from output when extended
            let selector = strip_placeholders(&selector);
            if selector.is_empty() {
                return; // Placeholder was not extended → invisible
            }
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

// ── 响应式序列化 ─────────────────────────────────────────────────────────

/// 序列化状态：累积 CssStmt 并渲染为字符串
#[derive(Clone)]
struct SerializeState {
    options: Options,
    buffer: String,
}

impl SerializeState {
    fn new(options: Options) -> Self {
        Self {
            options,
            buffer: String::new(),
        }
    }

    /// 压入 CssStmt 并立即渲染到 buffer
    fn push(&mut self, stmt: CssStmt) {
        // 预pend charset 只在第一次
        if self.buffer.is_empty() && !self.options.suppress_charset {
            match self.options.style {
                OutputStyle::Expanded | OutputStyle::Nested => {
                    self.buffer.push_str(CHARSET);
                }
                OutputStyle::Compressed => {}
            }
        }
        serialize_stmt(&stmt, &self.options, &mut self.buffer, 0);
    }

    /// 渲染累积内容为 String
    fn render(&self) -> String {
        self.buffer.clone()
    }
}

/// 响应式序列化：CssStmt 流 → String 流
///
/// 使用 scan_map 累积 CssStmt 状态到 SerializeState，
/// 流结束时通过 take_last(1) 只输出最终渲染结果。
/// box_it() 在 stage 尾端调用一次。
pub fn serialize_stream(
    css_stream: CssStream,
    options: Options,
) -> OutputStream {
    css_stream
        .scan_map(SerializeState::new(options), |state, stmt| {
            state.push(stmt);
            state.clone()
        })
        .take_last(1)
        .map(|state| state.render())
        .box_it()
}



