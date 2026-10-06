//! Serializer — converts `CssStmt` stream into formatted CSS string stream.
//!
//! Uses `OutputStyle` to control formatting (Compressed / Expanded / Nested).
//! Each top-level `CssStmt` is emitted as its own `String` in the output
//! stream, allowing downstream consumers to stream-write or join as needed.

use std::sync::Arc;

use rxrust::prelude::*;
use tracing::info_span;

use crate::reactive::CssStream;

/// CSS output formatting style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStyle {
    /// Compressed: no whitespace except required separators.
    /// Example: `a{color:red}`
    Compressed,
    /// Expanded: indented, each declaration on its own line.
    /// Example: `a {\n  color: red;\n}\n`
    Expanded,
    /// Nested: like Expanded but with nested rules visually indented.
    /// Example: `.foo {\n  color: red;\n  .bar {\n    display: none;\n  }\n}\n`
    Nested,
}

impl Default for OutputStyle {
    fn default() -> Self {
        OutputStyle::Expanded
    }
}

/// Compiler options controlling serialization and behavior.
#[derive(Debug, Clone)]
pub struct Options {
    /// Output formatting style.
    pub style: OutputStyle,
    /// Indent width in spaces (default 2).
    pub indent_width: u8,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            style: OutputStyle::Expanded,
            indent_width: 2,
        }
    }
}

/// Serialize a `CssStream` into a stream of CSS string chunks.
///
/// Each emitted `String` represents one fully-formatted top-level CSS
/// statement (rule, media block, etc.). Subscribers can join these or
/// stream-write them to a writer.
///
/// Internally uses `collect::<Vec<_>>()` followed by `from_iter` to
/// preserve the reactive style while applying formatting.
pub fn serialize(stream: CssStream, options: &Options) -> SharedBoxedObservable<'static, String, std::convert::Infallible> {
    let span = info_span!("serializer", style = ?options.style);
    let _guard = span.enter();

    let style = options.style;
    let indent_width = options.indent_width;

    // Collect all statements, then emit formatted chunks
    stream
        .collect::<Vec<_>>()
        .flat_map(move |stmts| {
            Shared::from_iter(stmts.into_iter().map(move |stmt| {
                format_stmt(&stmt, style, indent_width, 0)
            }))
        })
        .box_it()
}

/// Serialize and join all chunks into a single String.
///
/// Terminal convenience for when the full CSS output is needed at once.
pub fn serialize_to_string(stream: CssStream, options: &Options) -> String {
    let span = info_span!("serialize_to_string", style = ?options.style);
    let _guard = span.enter();

    // Use collect() to aggregate all CSS chunks
    let collected = serialize(stream, options).collect::<Vec<_>>();
    let result: Arc<std::sync::Mutex<Option<Vec<String>>>> =
        Arc::new(std::sync::Mutex::new(None));
    let r = result.clone();
    collected.subscribe(move |chunks| {
        *r.lock().unwrap() = Some(chunks);
    });

    let guard = Arc::try_unwrap(result).unwrap().into_inner().unwrap();
    let strings: Vec<String> = guard.unwrap_or_default();

    let mut output = String::new();
    for chunk in strings.iter() {
        output.push_str(chunk);
    }
    output
}

// ────────────────── Formatting engine ──────────────────

fn format_stmt(
    stmt: &crate::reactive::CssStmt,
    style: OutputStyle,
    indent_width: u8,
    depth: usize,
) -> String {
    match style {
        OutputStyle::Compressed => format_compressed(stmt),
        OutputStyle::Expanded => format_expanded(stmt, indent_width, depth),
        OutputStyle::Nested => format_nested(stmt, indent_width, depth),
    }
}

/// Compressed formatting — minimal whitespace.
fn format_compressed(stmt: &crate::reactive::CssStmt) -> String {
    match stmt {
        crate::reactive::CssStmt::Decl { property, value } => {
            format!("{}:{}", property, value)
        }
        crate::reactive::CssStmt::Rule { selector, inner } => {
            let inner_css: String = inner.iter().map(format_compressed).collect::<Vec<_>>().join(";");
            format!("{}{{{};}}", selector, inner_css)
        }
        crate::reactive::CssStmt::Media { query, inner } => {
            let inner_css: String = inner.iter().map(format_compressed).collect::<Vec<_>>().join("");
            format!("@media{}{{{}}}", query, inner_css)
        }
        crate::reactive::CssStmt::Supports { query, inner } => {
            let inner_css: String = inner.iter().map(format_compressed).collect::<Vec<_>>().join("");
            format!("@supports{}{{{}}}", query, inner_css)
        }
    }
}

/// Expanded formatting — indented, each declaration on its own line.
fn format_expanded(stmt: &crate::reactive::CssStmt, indent_width: u8, depth: usize) -> String {
    let pad = " ".repeat((indent_width as usize) * depth);

    match stmt {
        crate::reactive::CssStmt::Decl { property, value } => {
            format!("{}{}: {};\n", pad, property, value)
        }
        crate::reactive::CssStmt::Rule { selector, inner } => {
            let mut s = format!("{}{} {{\n", pad, selector);
            for child in inner {
                s.push_str(&format_expanded(child, indent_width, depth + 1));
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        crate::reactive::CssStmt::Media { query, inner } => {
            let mut s = format!("{}@media {} {{\n", pad, query);
            for child in inner {
                s.push_str(&format_expanded(child, indent_width, depth + 1));
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        crate::reactive::CssStmt::Supports { query, inner } => {
            let mut s = format!("{}@supports {} {{\n", pad, query);
            for child in inner {
                s.push_str(&format_expanded(child, indent_width, depth + 1));
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
    }
}

/// Nested formatting — like Expanded but nested rules stay inline.
fn format_nested(stmt: &crate::reactive::CssStmt, indent_width: u8, depth: usize) -> String {
    // For now, nested == expanded (future: visual nesting of child rules)
    format_expanded(stmt, indent_width, depth)
}
