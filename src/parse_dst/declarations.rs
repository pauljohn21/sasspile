//! 声明 / 规则块解析: parse_declarations / parse_block / flush_rule / try_flush_variable.

use crate::ast::{Node, Token};
use tracing;

/// flush_rule — 从缓冲中取出 tokens,拆分成 selector + body 解析为 Rule 节点
pub(super) fn flush_rule_buffer(buffer: &mut Vec<Token>) -> Vec<Node> {
    let tokens = buffer.drain(..).collect::<Vec<_>>();

    if let Some(lbrace_idx) = tokens.iter().position(|t| *t == Token::LBrace) {
        let selector_tokens = &tokens[..lbrace_idx];
        let selector = selector_tokens
            .iter()
            .filter_map(|t| match t {
                Token::Ident(s) => Some(s.as_str()),
                Token::Dot => Some("."),
                Token::Hash => Some("#"),
                Token::Colon => Some(":"),
                Token::Interpolation(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let body_tokens = &tokens[lbrace_idx + 1..];
        let body = parse_declarations(body_tokens);

        vec![Node::Rule { selector, body }]
    } else {
        Vec::new()
    }
}

/// 尝试从 buffer 中解析顶层 `$var : value ;` 变量声明
pub(super) fn try_flush_variable(buffer: &mut Vec<Token>) -> Option<Node> {
    if buffer.len() < 4 {
        return None;
    }

    // 跳过前导 Whitespace / Newline,找到第一个有效 token
    let start = buffer
        .iter()
        .position(|t| !matches!(t, Token::Whitespace | Token::Newline))?;
    if start > 0 && buffer[start] != Token::Dollar {
        return None;
    }
    if buffer[start] != Token::Dollar {
        return None;
    }
    let var_name = match buffer.get(start + 1) {
        Some(Token::Ident(s)) => s.clone(),
        _ => return None,
    };

    // 跳过 Whitespace 找到 Colon
    let mut idx = start + 2;
    while idx < buffer.len() && buffer[idx] == Token::Whitespace {
        idx += 1;
    }
    if idx >= buffer.len() || buffer[idx] != Token::Colon {
        return None;
    }
    idx += 1;
    // 跳过 Whitespace 到值
    while idx < buffer.len() && buffer[idx] == Token::Whitespace {
        idx += 1;
    }
    if idx >= buffer.len() {
        return None;
    }

    // 寻找 Semicolon 作为声明结束
    let semi_rel = buffer[idx..]
        .iter()
        .position(|t| matches!(t, Token::Semicolon | Token::Newline))?;

    let value_end = idx + semi_rel;
    let value_tokens = &buffer[idx..value_end];

    if tracing::enabled!(tracing::Level::DEBUG) {
        tracing::debug!(?buffer, var_name, value_end, "try_flush_variable attempt");
    }

    let value = value_tokens
        .iter()
        .filter_map(|t| match t {
            Token::Ident(s) => Some(s.as_str()),
            Token::Number(s) => Some(s.as_str()),
            Token::String(s) => Some(s.as_str()),
            Token::LParen => Some("("),
            Token::RParen => Some(")"),
            Token::Comma => Some(","),
            Token::Dot => Some("."),
            Token::Hash => Some("#"),
            Token::Colon => Some(":"),
            Token::At => Some("@"),
            Token::Dollar => Some("$"),
            Token::Op(s) => Some(s.as_str()),
            Token::Char(_) => None,
            Token::Whitespace => Some(" "),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    // 检查是否同时含有 LBrace (不是合法变量)
    if buffer[..=value_end].contains(&Token::LBrace) {
        return None;
    }

    let _ = buffer.drain(..=value_end);
    Some(Node::Variable { name: var_name, value })
}

/// 合并负数 token 模式: ["-", unit?, number] → ["-numberunit"]
/// 处理 tokenizer 将 -5px 切成 Ident("-"), Ident("px"), Number("5") 的乱序情况
fn merge_negative_numbers(parts: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(parts.len());
    let mut i = 0;
    while i < parts.len() {
        if parts[i] == "-" && i + 1 < parts.len() {
            // 向后查找 number 和 unit,允许 unit 在 number 之前
            let mut num = None;
            let mut unit = None;
            let mut end = i + 1;
            for j in (i + 1)..parts.len().min(i + 3) {
                let p = &parts[j];
                if num.is_none() && p.chars().all(|c| c.is_ascii_digit()) {
                    num = Some(p.clone());
                    end = j + 1;
                } else if unit.is_none()
                    && p.chars().all(|c| c.is_alphabetic())
                    && p != "-"
                {
                    unit = Some(p.clone());
                    end = j + 1;
                }
            }
            if let Some(n) = num {
                result.push(format!("-{n}{}", unit.as_deref().unwrap_or("")));
                i = end;
                continue;
            }
        }
        result.push(parts[i].clone());
        i += 1;
    }
    result
}

/// 解析 tokens 为声明列表 (Property:Value; 形式)
pub fn parse_declarations(tokens: &[Token]) -> Vec<Node> {
    let mut decls = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        // 前置空白跳过
        while i < tokens.len() && matches!(tokens[i], Token::Whitespace) {
            i += 1;
        }
        let mut prop_parts = Vec::new();
        while i < tokens.len() {
            match &tokens[i] {
                Token::Ident(s) => prop_parts.push(s.clone()),
                Token::Dollar => prop_parts.push("$".to_string()),
                Token::Interpolation(s) => prop_parts.push(s.clone()),
                Token::String(s) => prop_parts.push(s.clone()),
                Token::Whitespace => break,
                _ => break,
            }
            i += 1;
        }
        let prop = prop_parts.join("");
        if prop.is_empty() {
            i += 1;
            continue;
        }

        // 跳过多余空白找到 Colon
        while i < tokens.len() && matches!(tokens[i], Token::Whitespace) {
            i += 1;
        }
        if i >= tokens.len() || !matches!(tokens[i], Token::Colon) {
            i += 1;
            continue;
        }
        i += 1; // 跳过 Colon
        // 收集值内含空白 (空格分隔的 list 字段)
        while i < tokens.len() && matches!(tokens[i], Token::Whitespace) {
            i += 1;
        }
        let mut value_parts = Vec::new();
        while i < tokens.len() {
            match &tokens[i] {
                Token::Semicolon | Token::Newline => break,
                Token::Ident(s) => {
                    // 修复 "-5px" 被 tokenizer 拆成 Ident("-") + Number("5") + Ident("px") 的问题
                    if s == "-"
                        && i + 1 < tokens.len()
                        && matches!(&tokens[i + 1], Token::Number(_))
                    {
                        // 读取后续 number 和可能的 unit
                        let start = i;
                        let mut combined = Vec::new();
                        // 读取至多 4 个 token: - number/ident unit 等
                        while i < tokens.len() && (i - start) < 4 {
                            match &tokens[i] {
                                Token::Ident(t) => {
                                    // 若前一个 是 "-" 且当前是纯字母 unit (如 "px"),先行缓存,
                                    // 等到下一段 Number 再组合为 "-{num}{unit}"
                                    if t.chars().all(|c| c.is_alphabetic())
                                        && combined.last().map(|p: &String| p.as_str()) == Some("-")
                                    {
                                        // 检查再下一个 token 是否为 Number
                                        if matches!(tokens.get(i + 1), Some(Token::Number(_))) {
                                            let unit = t.clone();
                                            combined.pop(); // 移除 "-"
                                            combined.push(format!("__NEG_UNIT__{unit}"));
                                            i += 1;
                                            continue;
                                        }
                                    }
                                    combined.push(t.clone());
                                }
                                Token::Number(s) => {
                                    // 若 prev 是 "__NEG_UNIT__xxx" 占位,组合为 "-{num}{unit}"
                                    let combined_str = if let Some(last) = combined.last() {
                                        last.strip_prefix("__NEG_UNIT__").map(|unit| format!("-{s}{unit}"))
                                    } else {
                                        None
                                    };
                                    if let Some(c) = combined_str {
                                        combined.pop();
                                        combined.push(c);
                                        i += 1;
                                        continue;
                                    }
                                    // 若 prev 是 "-",组合为 "-{num}" 负数
                                    if combined.last().map(|p: &String| p.as_str()) == Some("-") {
                                        combined.pop();
                                        combined.push(format!("-{s}"));
                                        i += 1;
                                        continue;
                                    }
                                    combined.push(s.clone());
                                }
                                _ => break,
                            }
                            i += 1;
                        }
                        value_parts.push(combined.join(""));
                        continue;
                    }
                    value_parts.push(s.clone());
                }
                Token::Number(s) => {
                    // 若 prev 是 "-",组合为 "-{num}" 负数
                    if value_parts.last().map(|p: &String| p.as_str()) == Some("-") {
                        value_parts.pop();
                        value_parts.push(format!("-{s}"));
                        i += 1;
                        continue;
                    }
                    value_parts.push(s.clone());
                }
                Token::String(s) => value_parts.push(format!("\"{s}\"")),
                Token::Dollar => value_parts.push("$".to_string()),
                Token::Interpolation(s) => value_parts.push(s.clone()),
                Token::Whitespace => value_parts.push(" ".to_string()),
                Token::LParen => value_parts.push("(".to_string()),
                Token::RParen => value_parts.push(")".to_string()),
                Token::Comma => value_parts.push(",".to_string()),
                Token::Dot => value_parts.push(".".to_string()),
                Token::Hash => value_parts.push("#".to_string()),
                _ => {}
            }
            i += 1;
        }
        let value = merge_negative_numbers(&value_parts).join("").trim().to_string();
        if !prop.is_empty() && !value.is_empty() {
            decls.push(Node::Declaration { prop, value });
        }
        if i < tokens.len() && !matches!(tokens[i], Token::Newline) {
            i += 1;
        }
    }
    decls
}

/// 解析规则块 body (遇到嵌套 { 时递归成子 Rule)
pub fn parse_block(tokens: &[Token]) -> Vec<Node> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        // 清空空白
        while i < tokens.len() && matches!(tokens[i], Token::Whitespace) {
            i += 1;
        }
        if i >= tokens.len() {
            break;
        }

        // 检测模式: selector { ... } 还是 prop:value; (声明)
        // 关键:在 selector 扫描阶段先找到最近的 LBrace / Colon / Semicolon 来判断结构
        let mut j = i;
        let mut found_colon = false;
        let mut found_lbrace = false;
        while j < tokens.len() {
            match tokens[j] {
                Token::LBrace if !found_colon => {
                    found_lbrace = true;
                    break;
                }
                Token::Colon if !found_lbrace => {
                    found_colon = true;
                    // 继续扫描,找到 Semicolon 结束声明
                }
                Token::Semicolon | Token::Newline | Token::RBrace if found_colon => break,
                Token::RBrace if !found_colon && !found_lbrace => break,
                _ => {}
            }
            j += 1;
        }
        // 确保至少消耗一个 token 避免死循环
        if j == i {
            j = i + 1;
        }

        if found_lbrace {
            // Rule: selector { ... body ... RBrace }
            let selector: String = tokens[i..j]
                .iter()
                .filter_map(|t| match t {
                    Token::Ident(s) => Some(s.as_str()),
                    Token::Dot => Some("."),
                    Token::Hash => Some("#"),
                    Token::String(s) => Some(s.as_str()),
                    Token::Interpolation(s) => Some(s.as_str()),
                    Token::Dollar => Some("$"),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");
            // 找到 RBrace
            let lbrace_rel = tokens[i..].iter().position(|t| *t == Token::LBrace).unwrap_or(0);
            let lbrace_abs = i + lbrace_rel;
            let rbrace_rel = tokens[lbrace_abs..].iter().rposition(|t| *t == Token::RBrace).unwrap_or(0);
            let rbrace_abs = lbrace_abs + rbrace_rel;
            let inner = &tokens[lbrace_abs + 1..rbrace_abs];
            let body = if inner.is_empty() {
                Vec::new()
            } else {
                parse_declarations(inner)
            };
            if !selector.is_empty() {
                out.push(Node::Rule { selector, body });
            }
            j = rbrace_abs + 1;
        } else if found_colon {
            // 声明块: 把到 Semicolon/Newline 为止的整段交给 parse_declarations
            let decls = parse_declarations(&tokens[i..j]);
            out.extend(decls);
        } else {
            // 无法识别的结构,跳过
        }

        i = j;
    }

    out
}


