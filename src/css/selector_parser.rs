//! 选择器解析器——将选择器字符串解析为 `Selector` AST。
//!
//! 设计原则：
//! - 公开 API `parse_selector` 消费 `&str`，返回 `Selector`（纯函数，无 `&mut` 参数）
//! - 内部 `Parser` 封装游标状态（`&mut self` 仅在内部，不跨函数传递）
//! - 字符累积用 peek+next 循环（Peekable 不消费非匹配字符）

use super::selector_ast::{
    Combinator, ComplexSelector, CompoundSelector, Selector, SimpleSelector,
};
use std::iter::Peekable;

/// 位置游标解析器——封装在内部，不作为函数参数传递。
struct Parser {
    chars: Peekable<std::vec::IntoIter<char>>,
}

impl Parser {
    fn new(s: &str) -> Self {
        Self {
            chars: s.chars().collect::<Vec<_>>().into_iter().peekable(),
        }
    }

    fn skip_ws(&mut self) {
        while self.chars.peek().is_some_and(|c| c.is_whitespace()) {
            self.chars.next();
        }
    }

    /// 解析顶层 Selector——逗号分隔列表。
    fn parse_selector(mut self) -> Selector {
        self.skip_ws();
        let mut complexes: Vec<ComplexSelector> = Vec::new();

        while self.chars.peek().is_some() {
            self.skip_ws();
            if self.chars.peek().is_none() {
                break;
            }
            if let Some(complex) = self.parse_complex() {
                complexes.push(complex);
            } else {
                break;
            }
            self.skip_ws();
            if self.chars.peek().is_some_and(|c| *c == ',') {
                self.chars.next();
            }
        }

        if complexes.is_empty() {
            // 降级：剩余字符串作为单个 Type
            let rest: String = self.chars.by_ref().collect();
            Selector(vec![ComplexSelector {
                compounds: vec![(None, CompoundSelector(vec![SimpleSelector::Type(rest)]))],
            }])
        } else {
            Selector(complexes)
        }
    }

    /// 解析复杂选择器——组合器分隔的复合选择器序列。
    fn parse_complex(&mut self) -> Option<ComplexSelector> {
        let mut compounds: Vec<(Option<Combinator>, CompoundSelector)> = Vec::new();
        let mut pending_combinator: Option<Combinator> = None;

        while self.chars.peek().is_some() {
            self.skip_ws();
            if self.chars.peek().is_none() {
                break;
            }

            // 组合器
            if let Some(c) = self.chars.peek() {
                if matches!(*c, '>' | '+' | '~') {
                    pending_combinator = match *c {
                        '>' => Some(Combinator::Child),
                        '+' => Some(Combinator::Adjacent),
                        '~' => Some(Combinator::Sibling),
                        _ => unreachable!(),
                    };
                    self.chars.next();
                    continue;
                }
                // 逗号——复杂选择器结束
                if *c == ',' {
                    break;
                }
            }

            // 复合选择器
            let compound = self.parse_compound()?;
            compounds.push((pending_combinator.take(), compound));

            // 空格可能是后代组合器（但逗号不算）
            let had_ws = self.skip_ws_count() > 0;
            if had_ws && self.chars.peek().is_some_and(|c| !matches!(*c, '>' | '+' | '~' | ',')) {
                pending_combinator = Some(Combinator::Descendant);
            }
        }

        (!compounds.is_empty()).then_some(ComplexSelector { compounds })
    }

    /// 解析复合选择器——无空格的简单选择器序列。
    fn parse_compound(&mut self) -> Option<CompoundSelector> {
        let mut simples: Vec<SimpleSelector> = Vec::new();

        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() || matches!(c, '>' | '+' | '~' | ',') {
                break;
            }

            let simple = match c {
                '*' => {
                    self.chars.next();
                    SimpleSelector::Universal
                }
                '.' | '#' | '%' => {
                    self.chars.next();
                    let name = self.take_ident();
                    match c {
                        '.' => SimpleSelector::Class(name),
                        '#' => SimpleSelector::Id(name),
                        '%' => SimpleSelector::Placeholder(name),
                        _ => unreachable!(),
                    }
                }
                '[' => self.parse_attribute()?,
                ':' => {
                    self.chars.next(); // 消费第一个 ':'
                    if self.chars.peek().is_some_and(|c| *c == ':') {
                        // 伪元素 ::name
                        self.chars.next();
                        let name = self.take_ident();
                        let arg = self.take_pseudo_arg();
                        SimpleSelector::PseudoElement { name, arg }
                    } else {
                        // 伪类 :name
                        let name = self.take_ident();
                        let arg = self.take_pseudo_arg();
                        SimpleSelector::PseudoClass { name, arg }
                    }
                }
                _ if c.is_ascii_alphabetic() || c == '_' || c == '-' => {
                    let name = self.take_type_with_ns();
                    SimpleSelector::Type(name)
                }
                _ => {
                    self.chars.next();
                    break;
                }
            };
            simples.push(simple);
        }

        (!simples.is_empty()).then_some(CompoundSelector(simples))
    }

    /// 消费标识符字符——peek + next 循环，不消费非标识符字符。
    fn take_ident(&mut self) -> String {
        let mut s = String::new();
        while self.chars.peek().is_some_and(|c| {
            c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '\\'
        }) {
            s.push(self.chars.next().unwrap());
        }
        s
    }

    /// 消费类型选择器（含命名空间 `ns|type` 或 `ns|*`）。
    fn take_type_with_ns(&mut self) -> String {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            match c {
                c if c.is_ascii_alphanumeric() || c == '_' || c == '-' => {
                    s.push(c);
                    self.chars.next();
                }
                '|' => {
                    s.push(c);
                    self.chars.next();
                    match self.chars.peek() {
                        Some(&'*') => {
                            s.push('*');
                            self.chars.next();
                        }
                        _ => {
                            // peek+next 循环收集命名空间类型名
                            while self.chars.peek().is_some_and(|c| {
                                c.is_ascii_alphanumeric() || *c == '_' || *c == '-'
                            }) {
                                s.push(self.chars.next().unwrap());
                            }
                        }
                    }
                }
                _ => break,
            }
        }
        s
    }

    /// 消费伪类/伪元素参数 `(...)`。
    fn take_pseudo_arg(&mut self) -> Option<String> {
        if !self.chars.peek().is_some_and(|c| *c == '(') {
            return None;
        }
        self.chars.next(); // 跳过 (
        let mut depth = 1i32;
        let mut arg = String::new();
        while self.chars.peek().is_some_and(|c| {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            depth > 0
        }) {
            arg.push(self.chars.next().unwrap());
        }
        if self.chars.peek().is_some_and(|c| *c == ')') {
            self.chars.next();
        }
        let trimmed = arg.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    }

    /// 解析属性选择器 `[name=value mod]`。
    fn parse_attribute(&mut self) -> Option<SimpleSelector> {
        self.chars.next(); // 跳过 [
        self.skip_ws();

        let mut name = String::new();
        while self.chars.peek().is_some_and(|c| {
            *c != ']' && *c != '=' && *c != '~' && *c != '|' && *c != '^' && *c != '$'
                && *c != '*'
                && !c.is_whitespace()
        }) {
            name.push(self.chars.next().unwrap());
        }
        self.skip_ws();

        let op = self.take_attr_op();
        self.skip_ws();

        let value = self.take_attr_value();
        self.skip_ws();

        let modifier = if self.chars.peek().is_some_and(|c| *c != ']') {
            let m = self.chars.next().unwrap().to_string();
            self.skip_ws();
            Some(m)
        } else {
            None
        };

        if self.chars.peek().is_some_and(|c| *c == ']') {
            self.chars.next();
        }

        Some(SimpleSelector::Attribute {
            name,
            op: op.filter(|s| !s.is_empty()),
            value: value.filter(|s| !s.is_empty()),
            modifier,
        })
    }

    /// 消费属性操作符（~=, |=, ^=, $=, *=, =）。
    fn take_attr_op(&mut self) -> Option<String> {
        match self.chars.peek() {
            Some(c) if matches!(*c, '~' | '|' | '^' | '$' | '*') => {
                let first = *c;
                self.chars.next();
                let mut s = String::from(first);
                if self.chars.peek().is_some_and(|c| *c == '=') {
                    s.push('=');
                    self.chars.next();
                }
                Some(s)
            }
            Some(c) if *c == '=' => {
                self.chars.next();
                Some("=".to_string())
            }
            _ => None,
        }
    }

    /// 消费属性值（带引号或无引号）。
    fn take_attr_value(&mut self) -> Option<String> {
        match self.chars.peek() {
            Some(c) if *c == '"' || *c == '\'' => {
                let quote = *c;
                self.chars.next();
                let mut val = String::new();
                while self.chars.peek().is_some_and(|c| *c != quote) {
                    val.push(self.chars.next().unwrap());
                }
                if self.chars.peek().is_some_and(|c| *c == quote) {
                    self.chars.next();
                }
                Some(val)
            }
            Some(_) => {
                let mut val = String::new();
                while self.chars.peek().is_some_and(|c| *c != ']' && !c.is_whitespace()) {
                    val.push(self.chars.next().unwrap());
                }
                Some(val)
            }
            _ => None,
        }
    }

    /// 跳过空白，返回是否消费了字符。
    fn skip_ws_count(&mut self) -> usize {
        let mut count = 0;
        while self.chars.peek().is_some_and(|c| c.is_whitespace()) {
            self.chars.next();
            count += 1;
        }
        count
    }
}

// ─── 公开 API：消费 &str → 返回 Selector（纯函数） ──────────────

/// 将选择器字符串解析为 `Selector` AST。
#[tracing::instrument(level = "debug", fields(sel = %input))]
pub fn parse_selector(input: &str) -> Selector {
    let input = input.trim();
    if input.is_empty() {
        return Selector(Vec::new());
    }
    Parser::new(input).parse_selector()
}
