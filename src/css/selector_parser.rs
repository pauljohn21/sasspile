//! 选择器解析器——将选择器字符串解析为 `Selector` AST。
//!
//! 设计原则：
//! - 公开 API `parse_selector` 消费 `&str`，返回 `Selector`（纯函数，无 `&mut` 参数）
//! - 内部 `Parser` 封装游标状态（`&mut self` 仅在内部，不跨函数传递）
//! - 字符累积用 peek+next 循环（Peekable 不消费非匹配字符）
//! - 所有条件分派用 `match`，禁止裸 `if`

use super::selector_ast::{
    Combinator, ComplexSelector, CompoundSelector, Namespace, Selector, SimpleSelector,
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
            match self.chars.peek() {
                None => break,
                Some(_) => match self.parse_complex() {
                    Some(complex) => complexes.push(complex),
                    None => break,
                },
            }
            self.skip_ws();
            match self.chars.peek() {
                Some(c) if *c == ',' => { self.chars.next(); }
                _ => {}
            }
        }

        match complexes.is_empty() {
            true => {
                // 降级：剩余字符串作为单个 Type（无命名空间）
                let rest: String = self.chars.by_ref().collect();
                Selector(vec![ComplexSelector {
                    compounds: vec![(
                        None,
                        CompoundSelector(vec![SimpleSelector::Type {
                            namespace: Namespace::None,
                            name: rest,
                        }]),
                    )],
                }])
            }
            false => Selector(complexes),
        }
    }

    /// 解析复杂选择器——组合器分隔的复合选择器序列。
    ///
    /// 特殊处理：
    /// - 尾随组合器（如 `.c +`）→ 追加空 compound 保留组合器
    /// - 多连续组合器（如 `> + .c`、`.c ~ ~ .d`）→ 追加空 compound 保留每个组合器
    fn parse_complex(&mut self) -> Option<ComplexSelector> {
        let mut compounds: Vec<(Option<Combinator>, CompoundSelector)> = Vec::new();
        let mut pending_combinator: Option<Combinator> = None;

        while self.chars.peek().is_some() {
            self.skip_ws();
            match self.chars.peek() {
                None => break,
                Some(c) => match *c {
                    '>' | '+' | '~' => {
                        // 如果已有 pending_combinator，说明是连续组合器
                        // 追加空 compound 保留前一个组合器
                        if pending_combinator.is_some() {
                            compounds.push((pending_combinator.take(), CompoundSelector(Vec::new())));
                        }
                        pending_combinator = match c {
                            '>' => Some(Combinator::Child),
                            '+' => Some(Combinator::Adjacent),
                            '~' => Some(Combinator::Sibling),
                            _ => unreachable!(),
                        };
                        self.chars.next();
                        continue;
                    }
                    ',' => break,
                    _ => {}
                },
            }

            // 复合选择器
            let compound = self.parse_compound()?;
            compounds.push((pending_combinator.take(), compound));

            // 空格可能是后代组合器（但逗号不算）
            let had_ws = self.skip_ws_count() > 0;
            match (had_ws, self.chars.peek()) {
                (true, Some(c)) if !matches!(*c, '>' | '+' | '~' | ',') => {
                    pending_combinator = Some(Combinator::Descendant);
                }
                _ => {}
            }
        }

        // 处理尾随组合器：如果循环结束时仍有 pending_combinator，追加空 compound
        if let Some(comb) = pending_combinator.take() {
            compounds.push((Some(comb), CompoundSelector(Vec::new())));
        }

        (!compounds.is_empty()).then_some(ComplexSelector { compounds })
    }

    /// 解析复合选择器——无空格的简单选择器序列。
    fn parse_compound(&mut self) -> Option<CompoundSelector> {
        let mut simples: Vec<SimpleSelector> = Vec::new();

        while let Some(&c) = self.chars.peek() {
            match c {
                _ if c.is_whitespace() || matches!(c, '>' | '+' | '~' | ',') => break,
                '|' => {
                    // 空命名空间 |type
                    self.chars.next(); // 消费 '|'
                    let name = match self.chars.peek() {
                        Some(&'*') => {
                            self.chars.next();
                            "*".to_string()
                        }
                        _ => self.take_ident(),
                    };
                    simples.push(SimpleSelector::Type {
                        namespace: Namespace::Empty,
                        name,
                    });
                }
                '*' => {
                    self.chars.next();
                    // 检查是否是 `*|type`（any namespace + type）或 `*|*`（any namespace + universal）
                    match self.chars.peek() {
                        Some(&'|') => {
                            self.chars.next(); // 消费 '|'
                            let name = match self.chars.peek() {
                                Some(&'*') => {
                                    self.chars.next();
                                    "*".to_string()
                                }
                                _ => self.take_ident(),
                            };
                            simples.push(SimpleSelector::Type {
                                namespace: Namespace::Any,
                                name,
                            });
                        }
                        _ => {
                            simples.push(SimpleSelector::Universal);
                        }
                    }
                }
                '&' => {
                    self.chars.next();
                    simples.push(SimpleSelector::ParentReference);
                }
                '.' | '#' | '%' => {
                    self.chars.next();
                    let name = self.take_ident();
                    let simple = match c {
                        '.' => SimpleSelector::Class(name),
                        '#' => SimpleSelector::Id(name),
                        '%' => SimpleSelector::Placeholder(name),
                        _ => unreachable!(),
                    };
                    simples.push(simple);
                }
                '[' => {
                    let attr = self.parse_attribute()?;
                    simples.push(attr);
                }
                ':' => {
                    self.chars.next(); // 消费第一个 ':'
                    match self.chars.peek() {
                        Some(c) if *c == ':' => {
                            // 伪元素 ::name
                            self.chars.next();
                            let name = self.take_ident();
                            let arg = self.take_pseudo_arg();
                            simples.push(SimpleSelector::PseudoElement { name, arg, is_class_syntax: false });
                        }
                        _ => {
                            // 可能是伪类 :name，也可能是单冒号伪元素（CSS 向后兼容）
                            let name = self.take_ident();
                            let arg = self.take_pseudo_arg();
                            // CSS 允许单冒号伪元素语法: :before, :after, :first-line, :first-letter
                            match name.as_str() {
                                "before" | "after" | "first-line" | "first-letter" => {
                                    simples.push(SimpleSelector::PseudoElement { name, arg, is_class_syntax: true });
                                }
                                _ => {
                                    simples.push(SimpleSelector::PseudoClass { name, arg });
                                }
                            }
                        }
                    }
                }
                _ if c.is_ascii_alphabetic() || c == '_' || c == '-' => {
                    let (namespace, name) = self.take_type_with_ns();
                    simples.push(SimpleSelector::Type { namespace, name });
                }
                _ => {
                    self.chars.next();
                    break;
                }
            }
        }

        (!simples.is_empty()).then_some(CompoundSelector(simples))
    }

    /// 消费标识符字符——peek + next 循环，不消费非标识符字符。
    #[allow(clippy::expect_used)]
    fn take_ident(&mut self) -> String {
        let mut s = String::new();
        while self.chars.peek().is_some_and(|c| {
            c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '\\'
        }) {
            // peek 已确认 Some，next 必然返回 Some
            s.push(self.chars.next().expect("take_ident: peek matched"));
        }
        s
    }

    /// 消费类型选择器（含命名空间 `ns|type` 或 `ns|*`）。
    ///
    /// 返回 `(Namespace, name)` 对：
    /// - `svg|circle` → `(Namespace::Explicit("svg"), "circle")`
    /// - `|circle` → `(Namespace::Empty, "circle")`
    /// - `*|circle` → `(Namespace::Any, "circle")`
    /// - `circle` → `(Namespace::None, "circle")`
    /// - `ns|*` → `(Namespace::Explicit("ns"), "*")`
    /// - `*|*` → `(Namespace::Any, "*")`
    #[allow(clippy::expect_used)]
    fn take_type_with_ns(&mut self) -> (Namespace, String) {
        let prefix = self.take_ident();
        match self.chars.peek() {
            Some(&'|') => {
                self.chars.next(); // 消费 '|'
                // 判断前缀是 `*|`（Any）还是 `ns|`（Explicit/Empty）
                let namespace = match prefix.as_str() {
                    "*" => Namespace::Any,
                    "" => Namespace::Empty,
                    ns => Namespace::Explicit(ns.to_string()),
                };
                // 消费类型名（可能是 `*` 或标识符）
                let name = match self.chars.peek() {
                    Some(&'*') => {
                        self.chars.next();
                        "*".to_string()
                    }
                    _ => self.take_ident(),
                };
                (namespace, name)
            }
            _ => {
                // 无前缀类型
                (Namespace::None, prefix)
            }
        }
    }

    /// 消费伪类/伪元素参数 `(...)`。
    #[allow(clippy::expect_used)]
    fn take_pseudo_arg(&mut self) -> Option<String> {
        match self.chars.peek() {
            Some(c) if *c == '(' => {
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
                    // peek 已确认 Some（depth > 0 保证还有字符），next 必然返回 Some
                    arg.push(self.chars.next().expect("take_pseudo_arg: depth > 0"));
                }
                match self.chars.peek() {
                    Some(c) if *c == ')' => { self.chars.next(); }
                    _ => {}
                }
                let trimmed = arg.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            }
            _ => None,
        }
    }

    /// 解析属性选择器 `[name=value mod]`。
    #[allow(clippy::expect_used)]
    fn parse_attribute(&mut self) -> Option<SimpleSelector> {
        self.chars.next(); // 跳过 [
        self.skip_ws();

        let mut name = String::new();
        while self.chars.peek().is_some_and(|c| {
            *c != ']' && *c != '=' && *c != '~' && *c != '|' && *c != '^' && *c != '$'
                && *c != '*'
                && !c.is_whitespace()
        }) {
            // peek 已确认 Some，next 必然返回 Some
            name.push(self.chars.next().expect("parse_attribute: peek matched"));
        }
        self.skip_ws();

        let op = self.take_attr_op();
        self.skip_ws();

        let value = self.take_attr_value();
        self.skip_ws();

        let modifier = match self.chars.peek() {
            Some(c) if *c != ']' => {
                // peek 已确认 Some，next 必然返回 Some
                let m = self.chars.next().expect("parse_attribute: modifier after ] check").to_string();
                self.skip_ws();
                Some(m)
            }
            _ => None,
        };

        match self.chars.peek() {
            Some(c) if *c == ']' => { self.chars.next(); }
            _ => {}
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
                match self.chars.peek() {
                    Some(c) if *c == '=' => {
                        s.push('=');
                        self.chars.next();
                    }
                    _ => {}
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
    #[allow(clippy::expect_used)]
    fn take_attr_value(&mut self) -> Option<String> {
        match self.chars.peek() {
            Some(c) if *c == '"' || *c == '\'' => {
                let quote = *c;
                self.chars.next();
                let mut val = String::new();
                while self.chars.peek().is_some_and(|c| *c != quote) {
                    // peek 已确认 Some（非 quote），next 必然返回 Some
                    val.push(self.chars.next().expect("take_attr_value: peek matched"));
                }
                match self.chars.peek() {
                    Some(c) if *c == quote => { self.chars.next(); }
                    _ => {}
                }
                Some(val)
            }
            Some(_) => {
                let mut val = String::new();
                while self.chars.peek().is_some_and(|c| *c != ']' && !c.is_whitespace()) {
                    // peek 已确认 Some（非 ] 非 ws），next 必然返回 Some
                    val.push(self.chars.next().expect("take_attr_value: peek matched"));
                }
                Some(val)
            }
            None => None,
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
    match input.is_empty() {
        true => Selector(Vec::new()),
        false => Parser::new(input).parse_selector(),
    }
}
