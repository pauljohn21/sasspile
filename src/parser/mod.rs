mod state;
pub use state::{ParseError, ParserState};
use rxrust::prelude::*;
use std::sync::{Arc, Mutex};
use crate::types::*;

pub fn parse_stream(token_stream: TokenStream, scope_id: u64) -> AstStream {
    parse_stream_with_paths(token_stream, scope_id, Vec::new())
}

pub fn parse_stream_with_paths(token_stream: TokenStream, scope_id: u64, include_paths: Vec<std::path::PathBuf>) -> AstStream {
    let source = Arc::new(std::sync::Mutex::new(Vec::<Token>::new()));
    let src = source.clone();
    let collected = token_stream.collect::<Vec<_>>();
    collected.subscribe(move |toks| { *src.lock().unwrap() = toks; });
    let guard = source.lock().unwrap();
    let tokens = guard.clone();
    drop(guard);
    let mut ps = ParserState::with_include_paths(include_paths);
    for t in tokens { ps.push_token(t); }
    let nodes = parse_all_nodes(&mut ps, scope_id);
    Shared::from_iter(nodes).box_it()
}

fn parse_all_nodes(ps: &mut ParserState, scope_id: u64) -> Vec<AstNode> {
    let mut result = Vec::new();
    while let Some(tok) = ps.peek() {
        match tok {
            Token::Dollar => {
                if let Some(node) = parse_variable_decl(ps, scope_id) {
                    result.push(node);
                } else { ps.next_token(); }
            }
            Token::AtMedia | Token::AtSupports | Token::AtIf | Token::AtElse |
            Token::AtFor | Token::AtEach | Token::AtWhile | Token::AtMixin |
            Token::AtInclude | Token::AtFunction | Token::AtReturn | Token::AtUse |
            Token::AtForward | Token::AtImport | Token::AtExtend | Token::AtAtRoot |
            Token::AtContent | Token::AtWarn | Token::AtDebug | Token::AtError |
            Token::AtCharset | Token::AtNamespace | Token::AtKeyframes |
            Token::AtFontFace | Token::AtPage | Token::AtCustomMedia |
            Token::AtCustomSelector | Token::AtName | Token::IdentAt(_) => {
                if let Some(node) = parse_at_rule(ps, scope_id) {
                    result.push(node);
                } else { ps.next_token(); }
            }
            Token::Ident(_) => {
                if is_style_decl(ps) {
                    if let Some(node) = parse_style_decl(ps) {
                        result.push(node);
                    } else { ps.next_token(); }
                } else if let Some(node) = parse_rule(ps, scope_id) {
                    result.push(node);
                } else { ps.next_token(); }
            }
            Token::Dot | Token::HashId(_) | Token::Ampersand |
            Token::LBracket | Token::Colon | Token::Star => {
                if let Some(node) = parse_rule(ps, scope_id) {
                    result.push(node);
                } else { ps.next_token(); }
            }
            Token::Eof | Token::RBrace => break,
            _ => { ps.next_token(); }
        }
    }
    result
}

fn is_style_decl(ps: &ParserState) -> bool {
    if matches!(ps.peek(), Some(Token::Ident(_))) {
        matches!(ps.peek_n(1), Some(Token::Colon))
    } else {
        false
    }
}

fn parse_style_decl(ps: &mut ParserState) -> Option<AstNode> {
    skip_whitespace(ps);
    let property = match ps.next_token() {
        Some(Token::Ident(n)) => n,
        _ => return None,
    };
    if !matches!(ps.peek(), Some(Token::Colon)) {
        return None;
    }
    ps.next_token();
    let value = parse_value(ps)?;
    if matches!(ps.peek(), Some(Token::Semicolon)) {
        ps.next_token();
    }
    Some(AstNode::StyleDecl { property, value: Box::new(value) })
}

fn parse_variable_decl(ps: &mut ParserState, scope_id: u64) -> Option<AstNode> {
    ps.next_token(); // consume $
    let name = match ps.next_token() {
        Some(Token::Ident(n)) => n,
        _ => return None,
    };
    if !matches!(ps.peek(), Some(Token::Colon)) {
        return None;
    }
    ps.next_token();
    let value = parse_value(ps)?;
    if matches!(ps.peek(), Some(Token::Semicolon)) {
        ps.next_token();
    }
    Some(AstNode::VariableDecl { name, value: Box::new(value), scope_id })
}

fn parse_value(ps: &mut ParserState) -> Option<AstNode> {
    parse_expression(ps, 0)
}

/// Parse a comma-separated list for `@each` iteration.
/// Handles both implicit list (`a, b, c`) and single expression (`$map` or `(a, b)`).
fn parse_each_list(ps: &mut ParserState) -> Option<AstNode> {
    skip_whitespace(ps);
    let first = parse_expression(ps, 0)?;
    skip_whitespace(ps);
    if !matches!(ps.peek(), Some(Token::Comma)) {
        return Some(first);
    }
    let mut items = vec![first];
    while matches!(ps.peek(), Some(Token::Comma)) {
        ps.next_token(); // consume comma
        skip_whitespace(ps);
        items.push(parse_expression(ps, 0)?);
        skip_whitespace(ps);
    }
    Some(AstNode::ListLiteral(items))
}

/// Pratt-style expression parser with operator precedence.
/// `min_bp` is the minimum binding power to continue consuming infix operators.
fn parse_expression(ps: &mut ParserState, min_bp: u8) -> Option<AstNode> {
    let lhs = parse_atom(ps)?;

    // Collect space-separated atoms into an implicit space-list.
    // Example: `red green blue` → ListLiteral([red, green, blue])
    // Stop when we hit an infix operator, comma, closing paren, semicolon, etc.
    let lhs = if is_atom_start(ps.peek()) {
        let mut items = vec![lhs];
        while is_atom_start(ps.peek()) {
            match parse_atom(ps) {
                Some(atom) => items.push(atom),
                None => break,
            }
        }
        AstNode::ListLiteral(items)
    } else {
        lhs
    };

    let mut lhs = lhs;
    loop {
        skip_whitespace(ps);
        let op_token = ps.peek()?;
        let (left_bp, right_bp, bin_op) = match classify_infix(op_token) {
            Some(t) => t,
            None => break,
        };

        if left_bp < min_bp {
            break;
        }

        ps.next_token(); // consume operator

        let rhs = parse_expression(ps, right_bp)?;
        lhs = AstNode::BinOp {
            op: bin_op,
            left: Box::new(lhs),
            right: Box::new(rhs),
        };
    }

    Some(lhs)
}

/// Returns true if the token can start an atomic expression (same as what `parse_atom` accepts).
/// Excludes Sass grammar keywords (`from`, `through`, `to`, `in`) so that `@for`/`@each`
/// parsers can locate these delimiters after calling `parse_value`.
fn is_atom_start(tok: Option<&Token>) -> bool {
    match tok {
        // Identifiers that serve as Sass grammar keywords are NOT atom starters —
        // they terminate space-separated list collection to avoid consuming delimiters
        // like `from 1 through 3` or `$key in $map`.
        Some(Token::Ident(s)) => !matches!(s.as_str(), "from" | "through" | "to" | "in"),
        Some(Token::Number(..))
        | Some(Token::Str(_))
        | Some(Token::Dollar)
        | Some(Token::LParen)
        | Some(Token::InterpolationStart)
        | Some(Token::HashId(_)) => true,
        _ => false,
    }
}

/// Atomic expression: literal, variable, function call, paren group, unary, list.
fn parse_atom(ps: &mut ParserState) -> Option<AstNode> {
    skip_whitespace(ps);

    match ps.peek()? {
        Token::Number(n, unit) => {
            let n = *n;
            let u = unit.clone();
            ps.next_token();
            Some(AstNode::Literal(Value::Number(n, u)))
        }
        Token::Str(s) => {
            let s = s.clone();
            ps.next_token();
            Some(AstNode::Literal(Value::String(s)))
        }
        Token::Ident(s) => {
            let s = s.clone();
            ps.next_token();

            // Function call: name(...)
            if matches!(ps.peek(), Some(Token::LParen)) {
                return parse_function_call_with_name(ps, s);
            }

            // Keywords
            match s.as_str() {
                "true" => Some(AstNode::Literal(Value::Bool(true))),
                "false" => Some(AstNode::Literal(Value::Bool(false))),
                "null" => Some(AstNode::Literal(Value::Null)),
                "not" => {
                    // Unary 'not' operator
                    let expr = parse_expression(ps, 40)?;
                    Some(AstNode::UnaryOp { op: UnaryOp::Not, expr: Box::new(expr) })
                }
                "and" | "or" => {
                    // misparsed as ident by lexer in expression context — treat as Bool(true)
                    Some(AstNode::Literal(Value::String(s)))
                }
                other if other.starts_with('#') && other.len() == 7 => {
                    parse_hex_color(&other[1..])
                        .map(AstNode::Literal)
                        .or(Some(AstNode::Literal(Value::String(s))))
                }
                _ => Some(AstNode::Literal(Value::String(s))),
            }
        }
        Token::Dollar => {
            ps.next_token();
            match ps.next_token() {
                Some(Token::Ident(n)) => Some(AstNode::VariableRef { name: n, scope_id: 0 }),
                _ => None,
            }
        }
        Token::LParen => {
            // Parenthesized expression, list literal, or map literal
            ps.next_token();
            let inner = parse_expression(ps, 0)?;
            if matches!(ps.peek(), Some(Token::RParen)) {
                ps.next_token();
                Some(inner)
            } else if matches!(ps.peek(), Some(Token::Colon)) {
                // Map literal: ("key": value, "key2": value2)
                ps.next_token(); // consume ':'
                let val = parse_expression(ps, 0)?;
                let mut entries = vec![(key_to_string(&inner)?, val)];
                while matches!(ps.peek(), Some(Token::Comma)) {
                    ps.next_token();
                    let k_expr = parse_expression(ps, 0)?;
                    if matches!(ps.peek(), Some(Token::Colon)) {
                        ps.next_token();
                    }
                    let v_expr = parse_expression(ps, 0)?;
                    entries.push((key_to_string(&k_expr)?, v_expr));
                }
                if matches!(ps.peek(), Some(Token::RParen)) {
                    ps.next_token();
                }
                Some(AstNode::MapLiteral(entries))
            } else {
                // List literal: (a, b, c)
                let mut items = vec![inner];
                while matches!(ps.peek(), Some(Token::Comma)) {
                    ps.next_token();
                    items.push(parse_expression(ps, 0)?);
                }
                if matches!(ps.peek(), Some(Token::RParen)) {
                    ps.next_token();
                }
                Some(AstNode::ListLiteral(items))
            }
        }
        Token::InterpolationStart => {
            // #{...} interpolation in expression context
            ps.next_token(); // consume #{
            let inner = parse_expression(ps, 0)?;
            if matches!(ps.peek(), Some(Token::InterpolationEnd)) {
                ps.next_token(); // consume }
            }
            Some(AstNode::Interpolation(vec![inner]))
        }
        Token::Minus => {
            // Unary minus
            ps.next_token();
            let expr = parse_expression(ps, 50)?;
            Some(AstNode::UnaryOp { op: UnaryOp::Neg, expr: Box::new(expr) })
        }
        _ => None,
    }
}

/// Convert an AST expression to a map key string.
/// Accepts String literals, Number literals, or VariableRefs (use name).
fn key_to_string(expr: &AstNode) -> Option<String> {
    match expr {
        AstNode::Literal(Value::String(s)) => Some(s.clone()),
        AstNode::Literal(Value::Number(n, _)) => Some(format!("{}", n)),
        AstNode::VariableRef { name, .. } => Some(name.clone()),
        _ => None,
    }
}

/// Parse function call given the name (already consumed).
fn parse_function_call_with_name(ps: &mut ParserState, name: String) -> Option<AstNode> {
    if !matches!(ps.peek(), Some(Token::LParen)) {
        return None;
    }
    ps.next_token(); // consume '('
    let mut args = Vec::new();

    loop {
        skip_whitespace(ps);
        match ps.peek() {
            Some(Token::RParen) => { ps.next_token(); break; }
            Some(Token::Comma) => { ps.next_token(); }
            _ => {
                if let Some(arg) = parse_expression(ps, 0) {
                    args.push(arg);
                } else {
                    break;
                }
            }
        }
    }
    Some(AstNode::FunctionCall { name, args })
}

/// Classify an infix operator token: returns (left_bp, right_bp, bin_op) or None.
fn classify_infix(op: &Token) -> Option<(u8, u8, BinOp)> {
    match op {
        Token::Or => Some((10, 11, BinOp::Or)),
        Token::And => Some((20, 21, BinOp::And)),
        Token::Eq => Some((30, 31, BinOp::Eq)),
        Token::Ne => Some((30, 31, BinOp::Ne)),
        Token::Lt => Some((40, 41, BinOp::Lt)),
        Token::Gt => Some((40, 41, BinOp::Gt)),
        Token::Le => Some((40, 41, BinOp::Le)),
        Token::Ge => Some((40, 41, BinOp::Ge)),
        Token::Plus => Some((50, 51, BinOp::Add)),
        Token::Minus => Some((50, 51, BinOp::Sub)),
        Token::Star => Some((60, 61, BinOp::Mul)),
        Token::Slash => Some((60, 61, BinOp::Div)),
        Token::Percent => Some((60, 61, BinOp::Mod)),
        _ => None,
    }
}

fn parse_hex_color(hex: &str) -> Option<Value> {
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Value::Color(r, g, b, 255))
    } else { None }
}

fn parse_rule(ps: &mut ParserState, scope_id: u64) -> Option<AstNode> {
    let selector = parse_selector(ps)?;
    expect_token(ps, &Token::LBrace)?;
    let inner = parse_all_nodes(ps, scope_id);
    expect_token(ps, &Token::RBrace)?;
    Some(AstNode::Rule { selector, inner })
}

fn parse_selector(ps: &mut ParserState) -> Option<String> {
    let mut parts = Vec::new();
    loop {
        match ps.peek() {
            Some(Token::LBrace) => break,
            Some(Token::Ident(s)) => { parts.push(s.clone()); ps.next_token(); }
            Some(Token::Dot) => { parts.push(".".into()); ps.next_token(); }
            Some(Token::HashId(s)) => { parts.push(format!("#{}", s)); ps.next_token(); }
            Some(Token::Ampersand) => { parts.push("&".into()); ps.next_token(); }
            Some(Token::Colon) => { parts.push(":".into()); ps.next_token(); }
            Some(Token::Star) => { parts.push("*".into()); ps.next_token(); }
            Some(Token::Minus) => { parts.push("-".into()); ps.next_token(); }
            Some(Token::LBracket) => {
                parts.push("[".into());
                ps.next_token();
                while !matches!(ps.peek(), Some(Token::RBracket)) {
                    if let Some(tok) = ps.peek() {
                        parts.push(format!("{:?}", tok));
                        ps.next_token();
                    } else { break; }
                }
                if matches!(ps.peek(), Some(Token::RBracket)) {
                    parts.push("]".into());
                    ps.next_token();
                }
            }
            Some(Token::InterpolationStart) => {
                // #{...} interpolation in selector
                ps.next_token(); // consume #{
                // Parse the inner expression — for selectors, typically a variable
                if let Some(Token::Dollar) = ps.peek() {
                    ps.next_token(); // consume $
                    if let Some(Token::Ident(var)) = ps.next_token() {
                        parts.push(format!("${}", var));
                    }
                } else {
                    // Skip unknown content until InterpolationEnd
                    while !matches!(ps.peek(), Some(Token::InterpolationEnd)) {
                        ps.next_token();
                    }
                }
                if matches!(ps.peek(), Some(Token::InterpolationEnd)) {
                    ps.next_token(); // consume }
                }
            }
            Some(Token::Semicolon) => break,
            _ => break,
        }
        skip_whitespace(ps);
    }
    if parts.is_empty() { None } else { Some(parts.join("")) }
}

fn skip_whitespace(ps: &mut ParserState) {
    while let Some(Token::Whitespace) = ps.peek() { ps.next_token(); }
}

fn parse_at_rule(ps: &mut ParserState, scope_id: u64) -> Option<AstNode> {
    let at_tok = ps.next_token()?;
    match at_tok {
        Token::AtMedia => {
            let query = parse_at_query(ps);
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let inner = parse_all_nodes(ps, scope_id);
            skip_whitespace(ps);
            expect_token(ps, &Token::RBrace)?;
            Some(AstNode::Media { query, inner })
        }
        Token::AtIf => {
            skip_whitespace(ps);
            let cond = parse_value(ps)?;
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let then_branch = parse_all_nodes(ps, scope_id);
            expect_token(ps, &Token::RBrace)?;
            let else_branch = match ps.peek() {
                Some(Token::AtElse) => {
                    ps.next_token();
                    if matches!(ps.peek(), Some(Token::AtIf)) {
                        let else_if_node = parse_at_rule(ps, scope_id)?;
                        Some(vec![else_if_node])
                    } else {
                        expect_token(ps, &Token::LBrace)?;
                        let eb = parse_all_nodes(ps, scope_id);
                        expect_token(ps, &Token::RBrace)?;
                        Some(eb)
                    }
                }
                _ => None,
            };
            Some(AstNode::If { cond: Box::new(cond), then_branch, else_branch })
        }
        Token::AtFor => {
            // @for $var from N to/through M { }
            skip_whitespace(ps);
            expect_token(ps, &Token::Dollar)?;
            let var = match ps.next_token() {
                Some(Token::Ident(n)) => n, _ => return None,
            };
            // consume 'from'
            skip_whitespace(ps);
            if matches!(ps.peek(), Some(Token::Ident(n)) if n == "from") { ps.next_token(); }
            let from = parse_value(ps)?;
            skip_whitespace(ps);
            let inclusive = if matches!(ps.peek(), Some(Token::Ident(n)) if n == "through") {
                ps.next_token(); true
            } else if matches!(ps.peek(), Some(Token::Ident(n)) if n == "to") {
                ps.next_token(); false
            } else { false };
            let to = parse_value(ps)?;
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let body = parse_all_nodes(ps, scope_id);
            skip_whitespace(ps);
            expect_token(ps, &Token::RBrace)?;
            Some(AstNode::For { var, from: Box::new(from), to: Box::new(to), inclusive, body })
        }
        Token::AtEach => {
            // @each $var in list { } OR @each $key, $value in $map { }
            skip_whitespace(ps);
            let mut vars = Vec::new();
            // Parse first variable
            expect_token(ps, &Token::Dollar)?;
            if let Some(Token::Ident(n)) = ps.next_token() { vars.push(n); }
            // Parse optional additional variables (comma-separated)
            skip_whitespace(ps);
            while matches!(ps.peek(), Some(Token::Comma)) {
                ps.next_token(); // consume comma
                skip_whitespace(ps);
                expect_token(ps, &Token::Dollar)?;
                if let Some(Token::Ident(n)) = ps.next_token() { vars.push(n); }
                skip_whitespace(ps);
            }
            // consume 'in'
            if matches!(ps.peek(), Some(Token::Ident(n)) if n == "in") { ps.next_token(); }
            // Parse comma-separated list: `a, b` or `(a, b)` or `$map`
            let list = parse_each_list(ps)?;
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let body = parse_all_nodes(ps, scope_id);
            expect_token(ps, &Token::RBrace)?;
            Some(AstNode::Each { vars, list: Box::new(list), body })
        }
        Token::AtWhile => {
            skip_whitespace(ps);
            let cond = parse_value(ps)?;
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let body = parse_all_nodes(ps, scope_id);
            expect_token(ps, &Token::RBrace)?;
            Some(AstNode::While { cond: Box::new(cond), body })
        }
        Token::AtMixin => {
            // @mixin name($params) { }
            skip_whitespace(ps);
            let name = match ps.next_token() {
                Some(Token::Ident(n)) => n, _ => return None,
            };
            skip_whitespace(ps);
            let params = if matches!(ps.peek(), Some(Token::LParen)) {
                parse_param_list(ps)
            } else { Vec::new() };
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let body = parse_all_nodes(ps, scope_id);
            skip_whitespace(ps);
            expect_token(ps, &Token::RBrace)?;
            Some(AstNode::MixinDecl { name, params, body })
        }
        Token::AtInclude => {
            // @include name(args) { @content }
            skip_whitespace(ps);
            let name = match ps.next_token() {
                Some(Token::Ident(n)) => n, _ => return None,
            };
            let args = if matches!(ps.peek(), Some(Token::LParen)) {
                parse_arg_list(ps)
            } else { Vec::new() };
            // Parse optional content block for @content substitution
            skip_whitespace(ps);
            let content = if matches!(ps.peek(), Some(Token::LBrace)) {
                ps.next_token(); // consume {
                let block = parse_all_nodes(ps, scope_id);
                if matches!(ps.peek(), Some(Token::RBrace)) {
                    ps.next_token(); // consume }
                }
                block
            } else {
                if matches!(ps.peek(), Some(Token::Semicolon)) {
                    ps.next_token();
                }
                Vec::new()
            };
            Some(AstNode::MixinCall { name, args, content })
        }
        Token::AtFunction => {
            skip_whitespace(ps);
            let name = match ps.next_token() {
                Some(Token::Ident(n)) => n, _ => return None,
            };
            skip_whitespace(ps);
            let params = if matches!(ps.peek(), Some(Token::LParen)) {
                parse_param_list(ps)
            } else { Vec::new() };
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let body = parse_all_nodes(ps, scope_id);
            skip_whitespace(ps);
            expect_token(ps, &Token::RBrace)?;
            Some(AstNode::FunctionDecl { name, params, body })
        }
        Token::AtReturn => {
            skip_whitespace(ps);
            let val = parse_value(ps)?;
            skip_whitespace(ps);
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next_token(); }
            Some(AstNode::Return(Box::new(val)))
        }
        Token::AtImport => {
            // @import "path";
            skip_whitespace(ps);
            let path = match ps.peek() {
                Some(Token::Str(s)) => { let s = s.clone(); ps.next_token(); s }
                Some(Token::Ident(s)) => { let s = s.clone(); ps.next_token(); s }
                _ => String::new(),
            };
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next_token(); }
            // Resolve the import path and inline the parsed nodes
            resolve_import(ps, &path)
        }
        Token::AtUse | Token::AtForward => {
            // @use "path" / @forward "path"
            skip_whitespace(ps);
            let _path = match ps.peek() {
                Some(Token::Str(s)) => { let s = s.clone(); ps.next_token(); s }
                Some(Token::Ident(s)) => { let s = s.clone(); ps.next_token(); s }
                _ => String::new(),
            };
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next_token(); }
            None // produce no CSS
        }
        Token::AtAtRoot => {
            // @at-root selector { }
            skip_whitespace(ps);
            let selector = parse_selector(ps).unwrap_or_default();
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let inner = parse_all_nodes(ps, scope_id);
            skip_whitespace(ps);
            expect_token(ps, &Token::RBrace)?;
            // Emit as special CSS marker for serializer to hoist
            Some(AstNode::Rule { selector: format!("/*@at-root*/ {}", selector), inner })
        }
        Token::AtError => {
            // @error "message"
            skip_whitespace(ps);
            let val = parse_value(ps).unwrap_or(AstNode::Literal(Value::Null));
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next_token(); }
            Some(AstNode::Warn(Box::new(val)))
        }
        Token::AtContent => {
            // @content inside mixin body — represented as AstNode::Content marker
            skip_whitespace(ps);
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next_token(); }
            Some(AstNode::Content)
        }
        Token::AtCharset | Token::AtNamespace | Token::AtCustomMedia |
        Token::AtCustomSelector | Token::AtPage | Token::AtFontFace |
        Token::AtKeyframes | Token::AtElse => {
            skip_whitespace(ps);
            // Consume until semicolon or block
            while !matches!(ps.peek(), Some(Token::Semicolon | Token::LBrace | Token::Eof)) {
                ps.next_token();
            }
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next_token(); }
            None // Phase 1: these produce deferred/unsupported CSS
        }
        Token::AtWarn => {
            skip_whitespace(ps);
            let val = parse_value(ps).unwrap_or(AstNode::Literal(Value::Null));
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next_token(); }
            Some(AstNode::Warn(Box::new(val)))
        }
        Token::AtDebug => {
            skip_whitespace(ps);
            let val = parse_value(ps).unwrap_or(AstNode::Literal(Value::Null));
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next_token(); }
            Some(AstNode::Debug(Box::new(val)))
        }
        Token::AtSupports => {
            let query = parse_at_query(ps);
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let inner = parse_all_nodes(ps, scope_id);
            skip_whitespace(ps);
            expect_token(ps, &Token::RBrace)?;
            Some(AstNode::Supports { query, inner })
        }
        _ => None,
    }
}

fn parse_at_query(ps: &mut ParserState) -> String {
    let mut parts = Vec::new();
    while let Some(tok) = ps.peek() {
        match tok {
            Token::LBrace => break,
            Token::Ident(s) => { parts.push(s.clone()); ps.next_token(); }
            Token::Str(s) => { parts.push(s.clone()); ps.next_token(); }
            Token::Number(n, u) => {
                parts.push(if let Some(unit) = u { format!("{}{}", n, unit) } else { format!("{}", n) });
                ps.next_token();
            }
            Token::Whitespace => { ps.next_token(); }
            Token::And => { parts.push("and".into()); ps.next_token(); }
            Token::Or => { parts.push("or".into()); ps.next_token(); }
            Token::Dollar => {
                // Variable reference in query: $var-name
                let mut var_ref = String::from("$");
                ps.next_token();
                if let Some(Token::Ident(var_name)) = ps.peek() {
                    var_ref.push_str(var_name);
                    ps.next_token();
                }
                parts.push(var_ref);
            }
            Token::HashId(s) => {
                parts.push(format!("#{}", s));
                ps.next_token();
            }
            Token::Plus => { parts.push("+".into()); ps.next_token(); }
            Token::Star => { parts.push("*".into()); ps.next_token(); }
            Token::Slash => { parts.push("/".into()); ps.next_token(); }
            Token::Dot => { parts.push(".".into()); ps.next_token(); }
            Token::Comma => { parts.push(",".into()); ps.next_token(); }
            Token::Percent => { parts.push("%".into()); ps.next_token(); }
            Token::Bang => { parts.push("!".into()); ps.next_token(); }
            Token::Gt | Token::Lt | Token::Eq | Token::Ge | Token::Le | Token::Ne => {
                let s = format!("{:?}", tok);
                parts.push(s);
                ps.next_token();
            }
            Token::LParen | Token::RParen | Token::Colon | Token::Minus | Token::Semicolon => {
                let clean = match tok {
                    Token::LParen => "(",
                    Token::RParen => ")",
                    Token::Colon => ":",
                    Token::Minus => "-",
                    Token::Semicolon => ";",
                    _ => "",
                };
                parts.push(clean.into());
                ps.next_token();
            }
            Token::LBracket | Token::RBracket => {
                let clean = match tok {
                    Token::LBracket => "[",
                    Token::RBracket => "]",
                    _ => "",
                };
                parts.push(clean.into());
                ps.next_token();
            }
            _ => break,
        }
    }
    parts.join(" ")
}

fn expect_token(ps: &mut ParserState, expected: &Token) -> Option<()> {
    use std::mem::discriminant;
    match ps.peek() {
        Some(t) if discriminant(t) == discriminant(expected) => { ps.next_token(); Some(()) }
        _ => None,
    }
}

fn parse_param_list(ps: &mut ParserState) -> Vec<Param> {
    let mut params = Vec::new();
    if matches!(ps.peek(), Some(Token::LParen)) { ps.next_token(); }
    loop {
        skip_whitespace(ps);
        match ps.peek() {
            Some(Token::Dollar) => {
                ps.next_token();
                if let Some(Token::Ident(n)) = ps.next_token() {
                    let default = if matches!(ps.peek(), Some(Token::Colon)) {
                        ps.next_token();
                        parse_value(ps).map(Box::new)
                    } else { None };
                    params.push(Param { name: n, default_value: default });
                }
            }
            Some(Token::RParen) => { ps.next_token(); break; }
            Some(Token::Comma) => { ps.next_token(); }
            _ => break,
        }
    }
    params
}

fn parse_arg_list(ps: &mut ParserState) -> Vec<AstNode> {
    let mut args = Vec::new();
    if matches!(ps.peek(), Some(Token::LParen)) { ps.next_token(); }
    loop {
        skip_whitespace(ps);
        match ps.peek() {
            Some(Token::RParen) => { ps.next_token(); break; }
            Some(Token::Comma) => { ps.next_token(); }
            _ => {
                if let Some(v) = parse_value(ps) { args.push(v); }
                else { break; }
            }
        }
    }
    args
}

/// Resolve an @import path and return parsed AST nodes from the imported file.
/// Returns Vec<AstNode> (possibly empty if file not found or cycle detected).
/// The caller inlines these nodes into the current parse output.
fn resolve_import_nodes(ps: &mut ParserState, path: &str) -> Vec<AstNode> {
    let mut candidates = import_candidates(path);

    // Also search in include_paths
    if !ps.include_paths.is_empty() {
        let extra: Vec<_> = ps.include_paths.iter()
            .flat_map(|base| {
                let full = base.join(path);
                import_candidates(full.to_str().unwrap_or(path))
            })
            .collect();
        candidates.extend(extra);
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|p| seen.insert(p.clone()));

    let found = candidates.into_iter().find(|p| p.exists());

    match found {
        Some(file_path) => {
            // Cycle detection
            if ps.imports_seen.contains(&file_path) {
                return Vec::new();
            }
            match std::fs::read_to_string(&file_path) {
                Ok(source) => {
                    parse_import_source(&source, ps, file_path)
                }
                Err(_) => Vec::new(),
            }
        }
        None => {
            tracing::warn!("@import path not found: {}", path);
            Vec::new()
        }
    }
}

/// Parse the source of an imported file with cycle tracking.
/// Includes a recursion depth guard to prevent stack overflow on pathological import chains.
fn parse_import_source(source: &str, parent_ps: &ParserState, file_path: std::path::PathBuf) -> Vec<AstNode> {
    thread_local! {
        static DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    DEPTH.with(|d| d.set(d.get() + 1));
    let depth = DEPTH.with(|d| d.get());
    if depth > 500 {
        tracing::error!("Import recursion depth {} exceeded limit at {:?}", depth, file_path);
        DEPTH.with(|d| d.set(d.get() - 1));
        return Vec::new();
    }

    let tokens = crate::lexer::scan(source);

    // Synchronously collect tokens via Mutex
    let tok_store: Arc<Mutex<Vec<Token>>> = Arc::new(Mutex::new(Vec::new()));
    let store_clone = tok_store.clone();
    let collected = tokens.collect::<Vec<_>>();
    collected.subscribe(move |toks| {
        *store_clone.lock().unwrap() = toks;
    });

    let all_toks = tok_store.lock().unwrap().clone();

    let mut child_ps = ParserState {
        include_paths: parent_ps.include_paths.clone(),
        imports_seen: parent_ps.imports_seen.clone(),
        ..ParserState::new()
    };
    child_ps.imports_seen.push(file_path);
    for t in all_toks {
        child_ps.push_token(t);
    }
    let result = parse_all_nodes(&mut child_ps, 0);

    DEPTH.with(|d| d.set(d.get() - 1));
    result
}

/// Generate candidate file paths for an @import path.
/// SCSS convention: `foo` → `_foo.scss`, `foo.scss`, `_foo.css`, `foo.css`
fn import_candidates(path: &str) -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();
    let p = std::path::Path::new(path);

    if p.extension().is_some() {
        candidates.push(p.to_path_buf());
    } else {
        candidates.push(p.with_extension("scss"));
        candidates.push(p.with_extension("css"));
    }

    // Partial convention: prefix with underscore
    if let Some(stem) = p.file_stem() {
        let stem_str = stem.to_string_lossy();
        if !stem_str.starts_with('_') {
            let parent = p.parent().unwrap_or(std::path::Path::new(""));
            let with_underscore = parent.join(format!("_{}", p.file_name().unwrap().to_string_lossy()));
            candidates.push(with_underscore.with_extension("scss"));
            candidates.push(with_underscore.with_extension("css"));
        }
    }

    candidates
}

/// Wrapper for @import that returns AstNode when used in parse_at_rule context.
/// Since imports may expand to multiple nodes, we wrap them in a Group-like node.
fn resolve_import(ps: &mut ParserState, path: &str) -> Option<AstNode> {
    let nodes = resolve_import_nodes(ps, path);
    if nodes.is_empty() {
        None
    } else {
        Some(AstNode::Import(nodes))
    }
}

