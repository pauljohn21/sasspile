//! Core compilation entry: AstNode stream → CssStmt stream → String.

use std::cell::RefCell;
use std::rc::Rc;

use rxrust::prelude::*;
use tracing::info_span;

use crate::reactive::{
    evaluate_to_css, pre_analysis, AstNode, AstStream, CompilerBus, EvalContext, ScopeId,
};

/// Compile a `Vec<AstNode>` into a `Vec<CssStmt>`.
pub fn compile_ast(items: Vec<AstNode>, scope_id: ScopeId) -> Vec<crate::reactive::CssStmt> {
    let span = info_span!("compile_ast", scope_id);
    let _guard = span.enter();

    let bus = CompilerBus::new();
    let ctx = Rc::new(EvalContext::new(bus, scope_id));

    let mut counter = 0;
    let analyzed = pre_analysis(items, scope_id, &mut counter);

    let stream: AstStream = Local::from_iter(analyzed).box_it_clone();
    let css_stream = evaluate_to_css(stream, ctx);

    // Use Rc<RefCell<>> to satisfy 'static bound on subscribe closure
    let result = Rc::new(RefCell::new(Vec::new()));
    let r = result.clone();
    css_stream.subscribe(move |stmt| r.borrow_mut().push(stmt));

    match Rc::try_unwrap(result) {
        Ok(cell) => cell.into_inner(),
        Err(_) => Vec::new(),
    }
}

/// Compile a Sass source string (placeholder — needs parser integration).
pub fn from_string(_source: &str) -> Result<String, crate::Error> {
    let _span = info_span!("from_string");
    Err(crate::Error::msg("from_string: not yet integrated with parser"))
}

/// Compile from a pre-built AST (used by tests and the parser integration).
pub fn from_string_ast(items: Vec<AstNode>) -> Result<String, crate::Error> {
    let css_stmts = compile_ast(items, 0);
    Ok(serialize(&css_stmts))
}

/// Serialize `CssStmt` items to a CSS string.
pub fn serialize(stmts: &[crate::reactive::CssStmt]) -> String {
    let mut out = String::new();
    for stmt in stmts {
        serialize_stmt(stmt, &mut out, 0);
    }
    out
}

fn serialize_stmt(stmt: &crate::reactive::CssStmt, out: &mut String, indent: usize) {
    let pad = "  ".repeat(indent);
    match stmt {
        crate::reactive::CssStmt::Decl { property, value } => {
            out.push_str(&format!("{}{}: {};\n", pad, property, value));
        }
        crate::reactive::CssStmt::Rule { selector, inner } => {
            out.push_str(&format!("{}{} {{\n", pad, selector));
            for s in inner {
                serialize_stmt(s, out, indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        crate::reactive::CssStmt::Media { query, inner } => {
            out.push_str(&format!("@{} {{\n", query));
            for s in inner {
                serialize_stmt(s, out, indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        crate::reactive::CssStmt::Supports { query, inner } => {
            out.push_str(&format!("@{} {{\n", query));
            for s in inner {
                serialize_stmt(s, out, indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
    }
}
