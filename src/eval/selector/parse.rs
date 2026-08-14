//! 选择器解析——将选择器字符串解析为结构化表示。

use crate::error::{Result, SassError};

// —— 数据结构 ——

/// 复合选择器（不含组合器）。
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundSelector {
    /// 类型选择器（div, span, *），None 表示未指定。
    pub element: Option<String>,
    /// 类选择器列表。
    pub classes: Vec<String>,
    /// ID 选择器列表。
    pub ids: Vec<String>,
    /// 属性选择器列表。
    pub attrs: Vec<AttrSelector>,
    /// 伪类/伪元素列表。
    pub pseudos: Vec<PseudoSelector>,
}

/// 属性选择器。
#[derive(Debug, Clone, PartialEq)]
pub struct AttrSelector {
    pub name: String,
    pub op: Option<String>, // =, ~=, |=, ^=, $=, *=
    pub value: Option<String>,
}

/// 伪类/伪元素。
#[derive(Debug, Clone, PartialEq)]
pub struct PseudoSelector {
    pub name: String,
    /// true for :pseudo, false for ::pseudo。
    pub is_class: bool,
    /// 参数（用于 :nth-child(), :not() 等）。
    pub argument: Option<String>,
}

/// 组合器。
#[derive(Debug, Clone, PartialEq)]
pub enum Combinator {
    Descendant, // " "
    Child,      // ">"
    Adjacent,   // "+"
    Sibling,    // "~"
}

/// 带组合器的复合选择器。
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundWithCombinator {
    pub compound: CompoundSelector,
    pub combinator: Option<Combinator>, // None for the first compound
}

/// 复杂选择器（由组合器连接的复合选择器链）。
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
    pub parts: Vec<CompoundWithCombinator>,
}

/// 选择器列表（逗号分隔）。
pub type SelectorList = Vec<ComplexSelector>;

// —— 解析函数 ——

/// 解析单个选择器（可能包含逗号）。
pub fn parse_selector_list(s: &str) -> Result<SelectorList> {
    let mut result = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        result.push(parse_complex_selector(part)?);
    }
    Ok(result)
}

/// 解析单个复杂选择器（不含逗号）。
#[allow(dead_code)]
pub fn parse_selector(s: &str) -> Result<ComplexSelector> {
    parse_complex_selector(s)
}

fn parse_complex_selector(s: &str) -> Result<ComplexSelector> {
    let mut parts = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // 解析组合器（第一个 compound 除外）
        let combinator = if parts.is_empty() {
            None
        } else {
            match chars[i] {
                '>' => {
                    i += 1;
                    Some(Combinator::Child)
                }
                '+' => {
                    i += 1;
                    Some(Combinator::Adjacent)
                }
                '~' => {
                    i += 1;
                    Some(Combinator::Sibling)
                }
                c if c.is_whitespace() => {
                    // 跳过所有空白
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    Some(Combinator::Descendant)
                }
                _ => None,
            }
        };

        // 跳过空白
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }

        // 解析复合选择器
        let compound = parse_compound_selector(&chars, &mut i)?;
        parts.push(CompoundWithCombinator {
            compound,
            combinator,
        });
    }

    Ok(ComplexSelector { parts })
}

fn parse_compound_selector(chars: &[char], i: &mut usize) -> Result<CompoundSelector> {
    let mut element = None;
    let mut classes = Vec::new();
    let mut ids = Vec::new();
    let mut attrs = Vec::new();
    let mut pseudos = Vec::new();

    while *i < chars.len() {
        match chars[*i] {
            // 类选择器
            '.' => {
                *i += 1;
                let name = parse_ident(chars, i);
                classes.push(name);
            }
            // ID 选择器
            '#' => {
                *i += 1;
                // 检查是否是插值 #{...}
                if *i < chars.len() && chars[*i] == '{' {
                    // 跳过插值，恢复位置
                    break;
                }
                let name = parse_ident(chars, i);
                ids.push(name);
            }
            // 属性选择器
            '[' => {
                let attr = parse_attr_selector(chars, i)?;
                attrs.push(attr);
            }
            // 伪类/伪元素
            ':' => {
                *i += 1;
                let is_class = if *i < chars.len() && chars[*i] == ':' {
                    *i += 1;
                    false // ::pseudo
                } else {
                    true // :pseudo
                };
                let name = parse_ident(chars, i);
                let argument = if *i < chars.len() && chars[*i] == '(' {
                    Some(parse_paren_args(chars, i)?)
                } else {
                    None
                };
                pseudos.push(PseudoSelector {
                    name,
                    is_class,
                    argument,
                });
            }
            // 通配符 *（必须单独处理，因为 parse_ident 不消费 *）
            '*' => {
                if element.is_none() {
                    *i += 1; // 消费 *
                    element = Some("*".to_string());
                } else {
                    break;
                }
            }
            // 类型选择器
            c if c.is_ascii_alphabetic() || !c.is_ascii() => {
                if element.is_none() {
                    let name = parse_ident(chars, i);
                    element = Some(name);
                } else {
                    break;
                }
            }
            // 组合器或结束
            _ => break,
        }
    }

    Ok(CompoundSelector {
        element,
        classes,
        ids,
        attrs,
        pseudos,
    })
}

fn parse_ident(chars: &[char], i: &mut usize) -> String {
    let mut name = String::new();
    while *i < chars.len() {
        let c = chars[*i];
        if c.is_alphanumeric() || c == '-' || c == '_' || !c.is_ascii() {
            name.push(c);
            *i += 1;
        } else {
            break;
        }
    }
    name
}

fn parse_attr_selector(chars: &[char], i: &mut usize) -> Result<AttrSelector> {
    *i += 1; // 跳过 [
    let name = parse_ident(chars, i);
    let mut op = None;
    let mut value = None;

    // 检查操作符
    if *i < chars.len() {
        match chars[*i] {
            '=' => {
                op = Some("=".to_string());
                *i += 1;
            }
            '~' if *i + 1 < chars.len() && chars[*i + 1] == '=' => {
                op = Some("~=".to_string());
                *i += 2;
            }
            '|' if *i + 1 < chars.len() && chars[*i + 1] == '=' => {
                op = Some("|=".to_string());
                *i += 2;
            }
            '^' if *i + 1 < chars.len() && chars[*i + 1] == '=' => {
                op = Some("^=".to_string());
                *i += 2;
            }
            '$' if *i + 1 < chars.len() && chars[*i + 1] == '=' => {
                op = Some("$=".to_string());
                *i += 2;
            }
            '*' if *i + 1 < chars.len() && chars[*i + 1] == '=' => {
                op = Some("*=".to_string());
                *i += 2;
            }
            _ => {}
        }
    }

    // 解析值
    if op.is_some() && *i < chars.len() {
        let mut val = String::new();
        while *i < chars.len() && chars[*i] != ']' {
            val.push(chars[*i]);
            *i += 1;
        }
        value = Some(
            val.trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string(),
        );
    }

    if *i < chars.len() && chars[*i] == ']' {
        *i += 1;
    }

    Ok(AttrSelector { name, op, value })
}

fn parse_paren_args(chars: &[char], i: &mut usize) -> Result<String> {
    *i += 1; // 跳过 (
    let mut depth = 1;
    let mut args = String::new();
    while *i < chars.len() && depth > 0 {
        match chars[*i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if depth > 0 {
            args.push(chars[*i]);
        }
        *i += 1;
    }
    Ok(args)
}

/// 解析单个选择器（兼容性包装）。
#[allow(dead_code)]
pub fn parse_single_selector(s: &str) -> Result<ComplexSelector> {
    parse_selector_list(s)?
        .into_iter()
        .next()
        .ok_or_else(|| SassError::Eval("无法解析选择器".into()))
}
