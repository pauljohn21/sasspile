//! 指令解析 — @import / @use / @forward / @mixin / @include / @if / @for / @each。

use crate::ast::{Node, Token};
use crate::parse_dst::{parse_block, parse_declarations};

/// 从 buffer 起始尝试解析顶层指令 (跳过前导空白找到 At)。成功时 drain 已消耗的 tokens。
pub(crate) fn try_flush_directive(buffer: &mut Vec<Token>) -> Option<Node> {
    // 跳过前导 Whitespace / Newline 找到第一个有效 token
    let start = buffer
        .iter()
        .position(|t| !matches!(t, Token::Whitespace | Token::Newline))?;
    if buffer[start] != Token::At || start + 1 >= buffer.len() {
        return None;
    }
    let instr = match &buffer[start + 1] {
        Token::Ident(s) => s.clone(),
        _ => return None,
    };

    // 计算 drain_end (相对于 buffer 起始)
    let drain_end = compute_directive_drain_end(&buffer[start..], &instr)? + start;

    let drained: Vec<_> = buffer.drain(..drain_end).collect();
    tracing::debug!(?instr, drain_end, "directive flushed");
    build_directive_node(&drained[start..], &instr)
}

/// 计算指定指令在 buffer 中的 drain 末尾位置
fn compute_directive_drain_end(buffer: &[Token], instr: &str) -> Option<usize> {
    match instr {
        "import" | "use" | "forward" => flush_import_drain_end(buffer),
        "mixin" | "include" | "if" | "for" | "each" => {
            let lbrace_rel = buffer.iter().position(|t| *t == Token::LBrace)?;
            let rbrace_offset = buffer[lbrace_rel..]
                .iter()
                .rposition(|t| *t == Token::RBrace)?;
            Some(lbrace_rel + rbrace_offset + 1)
        }
        _ => None,
    }
}

fn flush_import_drain_end(buffer: &[Token]) -> Option<usize> {
    // import/use/forward: 到 Semicolon / Newline 止
    let end = buffer[2..]
        .iter()
        .position(|t| matches!(t, Token::Semicolon | Token::Newline))?;
    Some(end + 2 + 1)
}

fn build_directive_node(tokens: &[Token], instr: &str) -> Option<Node> {
    match instr {
        "import" | "use" | "forward" => build_import_like(tokens, instr),
        "mixin" => build_mixin(tokens),
        "include" => build_include(tokens),
        "if" => build_if(tokens),
        "for" => build_for(tokens),
        "each" => Some(Node::Directive {
            name: "each".into(),
            args: String::new(),
        }),
        _ => None,
    }
}

fn build_import_like(tokens: &[Token], instr: &str) -> Option<Node> {
    let path = tokens
        .iter()
        .skip(2)
        .find_map(|t| match t {
            Token::String(s) => Some(s.clone()),
            Token::Ident(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default();

    match instr {
        "import" => Some(Node::Import { path }),
        "use" => Some(Node::Use { path }),
        "forward" => Some(Node::Forward { path }),
        _ => None,
    }
}

fn build_mixin(tokens: &[Token]) -> Option<Node> {
    if tokens.len() < 3 {
        return None;
    }
    let mut idx = 2;
    while matches!(tokens.get(idx), Some(Token::Whitespace)) {
        idx += 1;
    }
    let name = if matches!(tokens.get(idx), Some(Token::Ident(_))) {
        if let Some(Token::Ident(n)) = tokens.get(idx) {
            idx += 1;
            n.clone()
        } else {
            return None;
        }
    } else {
        String::new()
    };

    let params = if matches!(tokens.get(idx), Some(Token::LParen)) {
        parse_param_list(&tokens[idx..])
    } else {
        Vec::new()
    };

    let lbrace_idx = tokens.iter().position(|t| *t == Token::LBrace)?;
    let rbrace_idx = tokens.iter().rposition(|t| *t == Token::RBrace)?;
    let body_tokens = &tokens[lbrace_idx + 1..rbrace_idx];
    let body = parse_block(body_tokens);

    Some(Node::MixinDef {
        name,
        params,
        body,
    })
}

fn build_include(tokens: &[Token]) -> Option<Node> {
    let mut idx = 2;
    let name = if let Some(Token::Ident(n)) = tokens.get(idx) {
        idx += 1;
        n.clone()
    } else {
        return None;
    };

    let args = if matches!(tokens.get(idx), Some(Token::LParen)) {
        parse_arg_list_at(tokens, idx)
    } else {
        Vec::new()
    };

    Some(Node::MixinCall { name, args })
}

fn build_if(tokens: &[Token]) -> Option<Node> {
    if tokens.len() < 4 {
        return None;
    }
    let lbrace_rel = tokens.iter().position(|t| *t == Token::LBrace)?;
    let condition = tokens[2..lbrace_rel]
        .iter()
        .filter_map(|t| match t {
            Token::Ident(s) => Some(s.as_str()),
            Token::Number(s) => Some(s.as_str()),
            Token::Op(s) => Some(s.as_str()),
            Token::Char(c) => Some(Box::leak(c.to_string().into_boxed_str())),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();

    Some(Node::If {
        condition,
        then_branch: Vec::new(),
        else_branch: None,
    })
}

fn build_for(tokens: &[Token]) -> Option<Node> {
    if tokens.len() < 5 {
        return None;
    }
    let lbrace_rel = tokens.iter().position(|t| *t == Token::LBrace)?;
    let mut idx = 2;
    while matches!(tokens.get(idx), Some(Token::Whitespace)) {
        idx += 1;
    }
    if tokens.get(idx) != Some(&Token::Dollar) {
        return None;
    }
    idx += 1;
    while matches!(tokens.get(idx), Some(Token::Whitespace)) {
        idx += 1;
    }
    let var = if let Token::Ident(v) = tokens.get(idx)? {
        v.clone()
    } else {
        return None;
    };
    idx += 1;
    while matches!(tokens.get(idx), Some(Token::Whitespace)) {
        idx += 1;
    }
    let from_idx = tokens[idx..]
        .iter()
        .position(|t| matches!(t, Token::Ident(s) if s == "from"))
        .map(|p| p + idx)?;
    let to_idx = tokens[from_idx..]
        .iter()
        .position(|t| matches!(t, Token::Ident(s) if s == "to"))
        .map(|p| p + from_idx)?;

    let from = tokens[from_idx + 1..to_idx]
        .iter()
        .filter_map(|t| match t {
            Token::Number(s) => Some(s.as_str()),
            Token::Ident(s) => Some(s.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");
    let to = tokens[to_idx + 1..lbrace_rel]
        .iter()
        .filter_map(|t| match t {
            Token::Number(s) => Some(s.as_str()),
            Token::Ident(s) => Some(s.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    let rbrace_abs = tokens.iter().rposition(|t| *t == Token::RBrace)?;
    let body_tokens = &tokens[lbrace_rel + 1..rbrace_abs];
    let body = parse_block(body_tokens);

    Some(Node::For {
        var,
        from,
        to,
        body,
    })
}

/// 构建行内指令节点 (include / if / for / each)
pub(crate) fn build_inline_node(tokens: &[Token], instr: &str) -> Option<Node> {
    match instr {
        "include" => parse_include_tokens(tokens),
        "if" => parse_inline_if_body(tokens),
        "for" => parse_for_body(tokens),
        "each" => Some(Node::Directive {
            name: "each".into(),
            args: String::new(),
        }),
        _ => None,
    }
}

/// 解析 @include 调用 token 序列
fn parse_include_tokens(tokens: &[Token]) -> Option<Node> {
    if tokens.len() < 3 {
        return None;
    }
    let mut idx = 2;
    while matches!(tokens.get(idx), Some(Token::Whitespace)) {
        idx += 1;
    }
    let name = if let Token::Ident(n) = &tokens.get(idx)? {
        let name = n.clone();
        idx += 1;
        name
    } else {
        return None;
    };
    while matches!(tokens.get(idx), Some(Token::Whitespace)) {
        idx += 1;
    }
    let args = if matches!(tokens.get(idx), Some(Token::LParen)) {
        parse_arg_list_at(tokens, idx)
    } else {
        Vec::new()
    };
    Some(Node::MixinCall { name, args })
}

/// 解析 @if 指令: At Ident("if") cond LBrace body RBrace
pub(super) fn parse_inline_if_body(tokens: &[Token]) -> Option<Node> {
    if tokens.len() < 4 {
        return None;
    }
    let lbrace_rel = tokens.iter().position(|t| *t == Token::LBrace)?;
    let condition = tokens[2..lbrace_rel]
        .iter()
        .filter_map(|t| match t {
            Token::Ident(s) => Some(s.as_str()),
            Token::Number(s) => Some(s.as_str()),
            Token::Op(s) => Some(s.as_str()),
            Token::Char(c) => Some(Box::leak(c.to_string().into_boxed_str())),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string();

    let rbrace_rel = tokens.iter().rposition(|t| *t == Token::RBrace)?;
    let body_tokens = &tokens[lbrace_rel + 1..rbrace_rel];
    let body = parse_block(body_tokens);

    Some(Node::If {
        condition,
        then_branch: body,
        else_branch: None,
    })
}

/// 解析 @for 指令: At Ident("for") Dollar Ident(var) Ident("from") value Ident("to") value LBrace body RBrace
pub(super) fn parse_for_body(tokens: &[Token]) -> Option<Node> {
    if tokens.len() < 8 {
        return None;
    }
    let lbrace_rel = tokens.iter().position(|t| *t == Token::LBrace)?;

    let mut idx = 2;
    while matches!(tokens.get(idx), Some(Token::Whitespace)) {
        idx += 1;
    }
    if tokens.get(idx) != Some(&Token::Dollar) {
        return None;
    }
    idx += 1;
    while matches!(tokens.get(idx), Some(Token::Whitespace)) {
        idx += 1;
    }
    let var = if let Token::Ident(v) = tokens.get(idx)? {
        v.clone()
    } else {
        return None;
    };
    idx += 1;
    while matches!(tokens.get(idx), Some(Token::Whitespace)) {
        idx += 1;
    }
    let from_off = tokens[idx..lbrace_rel]
        .iter()
        .position(|t| matches!(t, Token::Ident(s) if s == "from"))
        .map(|p| p + idx)?;
    let to_off = tokens[from_off..lbrace_rel]
        .iter()
        .position(|t| matches!(t, Token::Ident(s) if s == "to"))
        .map(|p| p + from_off)?;

    let from = tokens[from_off + 1..to_off]
        .iter()
        .filter_map(|t| match t {
            Token::Number(s) => Some(s.as_str()),
            Token::Ident(s) => Some(s.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");
    let to = tokens[to_off + 1..lbrace_rel]
        .iter()
        .filter_map(|t| match t {
            Token::Number(s) => Some(s.as_str()),
            Token::Ident(s) => Some(s.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    let rbrace_rel = tokens.iter().rposition(|t| *t == Token::RBrace)?;
    let body_tokens = &tokens[lbrace_rel + 1..rbrace_rel];
    let body = parse_block(body_tokens);

    Some(Node::For { var, from, to, body })
}

/// 从 tokens[pos..] 开始解析参数列表 (LParen ... RParen)
pub(super) fn parse_arg_list_at(tokens: &[Token], mut pos: usize) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0u32;
    while pos < tokens.len() {
        match &tokens[pos] {
            Token::LParen => depth += 1,
            Token::RParen => {
                if depth > 0 {
                    depth -= 1;
                }
                let c = current.trim().to_string();
                if !c.is_empty() {
                    args.push(c);
                }
                break;
            }
            Token::Comma | Token::Whitespace if depth == 0 => {
                let c = current.trim().to_string();
                if !c.is_empty() {
                    args.push(c);
                }
                current = String::new();
            }
            Token::Ident(s) => current.push_str(s),
            Token::Number(s) => {
                // 如当前 buffer 以 "-" 结尾（负号）,不插入空格
                if !current.is_empty() && !current.ends_with('-') {
                    current.push(' ');
                }
                current.push_str(s);
            }
            Token::String(s) => {
                current.push('"');
                current.push_str(s);
                current.push('"');
            }
            Token::Dollar => current.push('$'),
            _ => {}
        }
        pos += 1;
    }
    args
}

/// 解析参数列表 (位于 LParen ... RParen 之间) -> name序列 (可能含 ":default")
/// 返回形如 ["$x", "$y:default"] 的参数签名
fn parse_param_list(tokens: &[Token]) -> Vec<String> {
    let mut params = Vec::new();
    let mut i = 1; // skip LParen
    while i < tokens.len() {
        match &tokens[i] {
            Token::Dollar => {
                if let Token::Ident(name) = &tokens.get(i + 1).unwrap_or(&Token::Semicolon) {
                    let mut param = format!("${name}");
                    let mut j = i + 2;
                    // 跳过 Whitespace,检查是否含 :default
                    while matches!(tokens.get(j), Some(Token::Whitespace)) {
                        j += 1;
                    }
                    if matches!(tokens.get(j), Some(Token::Colon)) {
                        // 收集 : 后的默认值
                        j += 1;
                        while matches!(tokens.get(j), Some(Token::Whitespace)) {
                            j += 1;
                        }
                        let mut default = String::new();
                        while let Some(tok) = tokens.get(j) {
                            match tok {
                                Token::RParen | Token::Comma => break,
                                Token::Whitespace => {
                                    if !default.is_empty() {
                                        default.push(' ');
                                    }
                                }
                                Token::Ident(s) => default.push_str(s),
                                Token::Number(s) => {
                                    if !default.is_empty() {
                                        default.push(' ');
                                    }
                                    default.push_str(s);
                                }
                                Token::Dollar => default.push('$'),
                                _ => {}
                            }
                            j += 1;
                        }
                        param.push(':');
                        param.push_str(default.trim());
                    }
                    params.push(param);
                    i = i + 2;
                    continue;
                }
            }
            Token::RParen => break,
            _ => {}
        }
        i += 1;
    }
    params
}

pub(super) fn parse_arg_list(tokens: &[Token], start: usize) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0u32;
    let mut i = start;
    while i < tokens.len() {
        match &tokens[i] {
            Token::LParen => depth += 1,
            Token::RParen => {
                if depth > 0 {
                    depth -= 1;
                }
                let c = current.trim().to_string();
                if !c.is_empty() {
                    args.push(c);
                }
                break;
            }
            Token::Comma | Token::Whitespace if depth == 0 => {
                let c = current.trim().to_string();
                if !c.is_empty() {
                    args.push(c);
                }
                current = String::new();
            }
            Token::Ident(s) => current.push_str(s),
            Token::Number(s) => {
                // 如当前 buffer 以 "-" 结尾（负号）,不插入空格
                if !current.is_empty() && !current.ends_with('-') {
                    current.push(' ');
                }
                current.push_str(s);
            }
            Token::String(s) => {
                current.push('"');
                current.push_str(s);
                current.push('"');
            }
            Token::Dollar => current.push('$'),
            _ => {}
        }
        i += 1;
    }
    args
}
