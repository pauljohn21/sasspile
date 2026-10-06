mod state;
pub use state::{ParseError, ParserState};
use rxrust::prelude::*;
use std::sync::Arc;
use crate::types::*;

pub fn parse_stream(token_stream: TokenStream, scope_id: u64) -> AstStream {
    let source = Arc::new(std::sync::Mutex::new(Vec::<Token>::new()));
    let src = source.clone();
    let collected = token_stream.collect::<Vec<_>>();
    collected.subscribe(move |toks| { *src.lock().unwrap() = toks; });
    let guard = source.lock().unwrap();
    let tokens = guard.clone();
    drop(guard);
    let mut ps = ParserState::new();
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
                } else { ps.next(); }
            }
            Token::AtMedia | Token::AtSupports | Token::AtIf | Token::AtFor |
            Token::AtEach | Token::AtWhile | Token::AtMixin | Token::AtInclude |
            Token::AtFunction | Token::AtReturn | Token::AtUse | Token::AtForward |
            Token::AtExtend | Token::AtWarn | Token::AtDebug | Token::AtName |
            Token::IdentAt(_) => {
                if let Some(node) = parse_at_rule(ps, scope_id) {
                    result.push(node);
                } else { ps.next(); }
            }
            Token::Ident(_) => {
                if is_style_decl(ps) {
                    if let Some(node) = parse_style_decl(ps) {
                        result.push(node);
                    } else { ps.next(); }
                } else if let Some(node) = parse_rule(ps, scope_id) {
                    result.push(node);
                } else { ps.next(); }
            }
            Token::Dot | Token::HashId(_) | Token::Ampersand |
            Token::LBracket | Token::Colon | Token::Star => {
                if let Some(node) = parse_rule(ps, scope_id) {
                    result.push(node);
                } else { ps.next(); }
            }
            Token::Eof | Token::RBrace => break,
            _ => { ps.next(); }
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
    let property = match ps.next() {
        Some(Token::Ident(n)) => n,
        _ => return None,
    };
    match ps.peek() {
        Some(Token::Colon) => { ps.next(); }
        _ => return None,
    };
    let value = parse_value(ps)?;
    match ps.peek() {
        Some(Token::Semicolon) => { ps.next(); }
        _ => {}
    };
    Some(AstNode::StyleDecl { property, value: Box::new(value) })
}

fn parse_variable_decl(ps: &mut ParserState, scope_id: u64) -> Option<AstNode> {
    ps.next(); // consume $
    let name = match ps.next() {
        Some(Token::Ident(n)) => n,
        _ => return None,
    };
    match ps.peek() {
        Some(Token::Colon) => ps.next(),
        _ => return None,
    };
    let value = parse_value(ps)?;
    match ps.peek() {
        Some(Token::Semicolon) => { ps.next(); }
        _ => {}
    };
    Some(AstNode::VariableDecl { name, value: Box::new(value), scope_id })
}

fn parse_value(ps: &mut ParserState) -> Option<AstNode> {
    skip_whitespace(ps);
    match ps.peek()? {
        Token::Number(n, unit) => {
            let n = *n;
            let u = unit.clone();
            ps.next();
            if u.is_some() {
                Some(AstNode::Literal(Value::String(format!("{}{}", n as i64, u.unwrap()))))
            } else {
                Some(AstNode::Literal(Value::Number(n)))
            }
        }
        Token::Str(s) => { let s = s.clone(); ps.next(); Some(AstNode::Literal(Value::String(s))) }
        Token::Ident(s) => {
            let s = s.clone();
            ps.next();
            if s == "true" { Some(AstNode::Literal(Value::Bool(true))) }
            else if s == "false" { Some(AstNode::Literal(Value::Bool(false))) }
            else if s == "null" { Some(AstNode::Literal(Value::Null)) }
            else if s.starts_with('#') && s.len() == 7 {
                parse_hex_color(&s[1..]).map(AstNode::Literal).or(Some(AstNode::Literal(Value::String(s))))
            } else { Some(AstNode::Literal(Value::String(s))) }
        }
        Token::Dollar => {
            ps.next();
            match ps.next() {
                Some(Token::Ident(n)) => Some(AstNode::VariableRef { name: n, scope_id: 0 }),
                _ => None,
            }
        }
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
            Some(Token::Ident(s)) => { parts.push(s.clone()); ps.next(); }
            Some(Token::Dot) => { parts.push(".".into()); ps.next(); }
            Some(Token::HashId(s)) => { parts.push(format!("#{}", s)); ps.next(); }
            Some(Token::Ampersand) => { parts.push("&".into()); ps.next(); }
            Some(Token::Colon) => { parts.push(":".into()); ps.next(); }
            Some(Token::LBracket) => {
                parts.push("[".into());
                ps.next();
                while !matches!(ps.peek(), Some(Token::RBracket)) {
                    if let Some(tok) = ps.peek() {
                        parts.push(format!("{:?}", tok));
                        ps.next();
                    } else { break; }
                }
                if matches!(ps.peek(), Some(Token::RBracket)) {
                    parts.push("]".into());
                    ps.next();
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
    while let Some(Token::Whitespace) = ps.peek() { ps.next(); }
}

fn parse_at_rule(ps: &mut ParserState, scope_id: u64) -> Option<AstNode> {
    let at_tok = ps.next()?;
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
            let else_branch = if matches!(ps.peek(), Some(Token::Ident(n)) if n == "@else") {
                ps.next();
                expect_token(ps, &Token::LBrace)?;
                let eb = parse_all_nodes(ps, scope_id);
                expect_token(ps, &Token::RBrace)?;
                Some(eb)
            } else { None };
            Some(AstNode::If { cond: Box::new(cond), then_branch, else_branch })
        }
        Token::AtFor => {
            // @for $var from N to/through M { }
            skip_whitespace(ps);
            expect_token(ps, &Token::Dollar)?;
            let var = match ps.next() {
                Some(Token::Ident(n)) => n, _ => return None,
            };
            // consume 'from'
            skip_whitespace(ps);
            if matches!(ps.peek(), Some(Token::Ident(n)) if n == "from") { ps.next(); }
            let from = parse_value(ps)?;
            skip_whitespace(ps);
            let inclusive = if matches!(ps.peek(), Some(Token::Ident(n)) if n == "through") {
                ps.next(); true
            } else if matches!(ps.peek(), Some(Token::Ident(n)) if n == "to") {
                ps.next(); false
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
            // @each $var in list { }
            skip_whitespace(ps);
            expect_token(ps, &Token::Dollar)?;
            let var = match ps.next() {
                Some(Token::Ident(n)) => n, _ => return None,
            };
            skip_whitespace(ps);
            if matches!(ps.peek(), Some(Token::Ident(n)) if n == "in") { ps.next(); }
            let list = parse_value(ps)?;
            skip_whitespace(ps);
            expect_token(ps, &Token::LBrace)?;
            let body = parse_all_nodes(ps, scope_id);
            expect_token(ps, &Token::RBrace)?;
            Some(AstNode::Each { vars: vec![var], list: Box::new(list), body })
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
            let name = match ps.next() {
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
            // @include name(args)
            skip_whitespace(ps);
            let name = match ps.next() {
                Some(Token::Ident(n)) => n, _ => return None,
            };
            let args = if matches!(ps.peek(), Some(Token::LParen)) {
                parse_arg_list(ps)
            } else { Vec::new() };
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next(); }
            Some(AstNode::MixinCall { name, args })
        }
        Token::AtFunction => {
            skip_whitespace(ps);
            let name = match ps.next() {
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
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next(); }
            Some(AstNode::Return(Box::new(val)))
        }
        Token::AtUse | Token::AtForward => {
            // @use "path" / @forward "path"
            skip_whitespace(ps);
            let _path = match ps.peek() {
                Some(Token::Str(s)) => { let s = s.clone(); ps.next(); s }
                Some(Token::Ident(s)) => { let s = s.clone(); ps.next(); s }
                _ => String::new(),
            };
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next(); }
            None // produce no CSS
        }
        Token::AtWarn => {
            skip_whitespace(ps);
            let val = parse_value(ps).unwrap_or(AstNode::Literal(Value::Null));
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next(); }
            Some(AstNode::Warn(Box::new(val)))
        }
        Token::AtDebug => {
            skip_whitespace(ps);
            let val = parse_value(ps).unwrap_or(AstNode::Literal(Value::Null));
            if matches!(ps.peek(), Some(Token::Semicolon)) { ps.next(); }
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
            Token::Ident(s) => { parts.push(s.clone()); ps.next(); }
            Token::Str(s) => { parts.push(s.clone()); ps.next(); }
            Token::Number(n, u) => {
                parts.push(if let Some(unit) = u { format!("{}{}", n, unit) } else { format!("{}", n) });
                ps.next();
            }
            Token::Whitespace => { ps.next(); }
            Token::LParen | Token::RParen | Token::Colon => {
                parts.push(format!("{:?}", tok));
                ps.next();
            }
            _ => break,
        }
    }
    parts.join(" ")
}

fn expect_token(ps: &mut ParserState, expected: &Token) -> Option<()> {
    use std::mem::discriminant;
    match ps.peek() {
        Some(t) if discriminant(t) == discriminant(expected) => { ps.next(); Some(()) }
        _ => None,
    }
}

fn parse_param_list(ps: &mut ParserState) -> Vec<Param> {
    let mut params = Vec::new();
    if matches!(ps.peek(), Some(Token::LParen)) { ps.next(); }
    loop {
        skip_whitespace(ps);
        match ps.peek() {
            Some(Token::Dollar) => {
                ps.next();
                if let Some(Token::Ident(n)) = ps.next() {
                    let default = if matches!(ps.peek(), Some(Token::Colon)) {
                        ps.next();
                        parse_value(ps).map(Box::new)
                    } else { None };
                    params.push(Param { name: n, default_value: default });
                }
            }
            Some(Token::RParen) => { ps.next(); break; }
            Some(Token::Comma) => { ps.next(); }
            _ => break,
        }
    }
    params
}

fn parse_arg_list(ps: &mut ParserState) -> Vec<AstNode> {
    let mut args = Vec::new();
    if matches!(ps.peek(), Some(Token::LParen)) { ps.next(); }
    loop {
        skip_whitespace(ps);
        match ps.peek() {
            Some(Token::RParen) => { ps.next(); break; }
            Some(Token::Comma) => { ps.next(); }
            _ => {
                if let Some(v) = parse_value(ps) { args.push(v); }
                else { break; }
            }
        }
    }
    args
}

