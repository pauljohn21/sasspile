//! calc 表达式 AST——类型化解析与简化。
//!
//! 设计原则：
//! - 公开 API `parse_calc_expr` 消费 `&str`，返回 `Option<CalcNode>`（纯函数）
//! - 内部 `Parser` 封装游标（`&mut self` 仅在内部，不跨函数传递）
//! - 字符累积用 peek+next 循环

use std::fmt;
use std::iter::Peekable;

/// calc AST 节点。
#[derive(Debug, Clone, PartialEq)]
pub enum CalcNode {
    /// 数字 + 可选单位：`1px`, `2.5`, `30deg`
    Number(f64, Option<String>),
    /// 运算：`left op right`
    Op {
        op: CalcOp,
        left: Box<CalcNode>,
        right: Box<CalcNode>,
    },
    /// CSS 函数调用：`min(...)`, `max(...)`, `var(--x)` 等
    Func {
        name: String,
        args: Vec<CalcNode>,
    },
    /// CSS 变量：`var(--name, fallback)`
    Var {
        name: String,
        fallback: Option<Box<CalcNode>>,
    },
}

/// calc 运算符。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// calc 简化错误。
#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    /// 不兼容的单位
    IncompatibleUnits(String, String),
    /// 除以零
    DivisionByZero,
    /// 无法简化（保留原样）
    CannotSimplify,
}

impl fmt::Display for CalcOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
        }
    }
}

impl fmt::Display for CalcNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n, unit) => format_number(f, *n, unit.as_deref()),
            Self::Op { op, left, right } => write!(f, "{left} {op} {right}"),
            Self::Func { name, args } => {
                let parts: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "{name}({})", parts.join(", "))
            }
            Self::Var { name, fallback } => {
                write!(f, "var({name}")?;
                if let Some(fb) = fallback {
                    write!(f, ", {fb}")?;
                }
                write!(f, ")")
            }
        }
    }
}

/// 格式化数字——避免显示 `-0`，整数不加 `.0`。
fn format_number(f: &mut fmt::Formatter<'_>, n: f64, unit: Option<&str>) -> fmt::Result {
    let n = if n == 0.0 { n.abs() } else { n };
    if n.fract() == 0.0 && n.abs() < 1e15 {
        write!(f, "{n:.0}")?;
    } else {
        write!(f, "{n}")?;
    }
    if let Some(u) = unit {
        write!(f, "{u}")?;
    }
    Ok(())
}

// ─── 解析器 ──────────────────────────────────────────────────────

/// 内部解析器——封装游标，`&mut self` 仅在内部。
struct Parser {
    chars: Peekable<std::vec::IntoIter<char>>,
    /// 缓冲区——用于 lookahead（peek 第二个字符）。
    buf: Vec<char>,
}

impl Parser {
    fn new(s: &str) -> Self {
        Self {
            chars: s.chars().collect::<Vec<_>>().into_iter().peekable(),
            buf: Vec::new(),
        }
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.advance();
        }
    }

    /// peek 当前字符。
    fn peek(&mut self) -> Option<char> {
        if self.buf.is_empty() {
            self.buf.push(self.chars.next()?);
        }
        Some(self.buf[0])
    }

    /// peek 第二个字符（lookahead 2）。
    fn peek2(&mut self) -> Option<char> {
        while self.buf.len() < 2 {
            self.buf.push(self.chars.next()?);
        }
        Some(self.buf[1])
    }

    /// 消费当前字符。
    fn advance(&mut self) -> Option<char> {
        if self.buf.is_empty() {
            self.chars.next()
        } else {
            let c = self.buf.remove(0);
            if !self.buf.is_empty() {
                // 从 chars 补一个到 buf
                if let Some(next) = self.chars.next() {
                    self.buf.push(next);
                }
            }
            Some(c)
        }
    }

    /// 是否还有字符。
    fn has_more(&mut self) -> bool {
        self.peek().is_some()
    }

    /// 解析表达式（加减法——最低优先级）。
    fn parse_expr(&mut self) -> Option<CalcNode> {
        let mut left = self.parse_term()?;
        loop {
            self.skip_ws();
            let op = match self.peek() {
                Some('+') => CalcOp::Add,
                Some('-') => CalcOp::Sub,
                _ => break,
            };
            self.advance();
            self.skip_ws();
            let right = self.parse_term()?;
            left = CalcNode::Op { op, left: Box::new(left), right: Box::new(right) };
        }
        Some(left)
    }

    /// 解析项（乘除法——高优先级）。
    fn parse_term(&mut self) -> Option<CalcNode> {
        let mut left = self.parse_factor()?;
        loop {
            self.skip_ws();
            let op = match self.peek() {
                Some('*') => CalcOp::Mul,
                Some('/') => CalcOp::Div,
                _ => break,
            };
            self.advance();
            self.skip_ws();
            let right = self.parse_factor()?;
            left = CalcNode::Op { op, left: Box::new(left), right: Box::new(right) };
        }
        Some(left)
    }

    /// 解析因子：数字、括号、函数、var()。
    fn parse_factor(&mut self) -> Option<CalcNode> {
        self.skip_ws();
        let c = self.peek()?;

        // 括号
        if c == '(' {
            self.advance();
            let inner = self.parse_expr()?;
            self.skip_ws();
            if self.peek().is_some_and(|c| c == ')') {
                self.advance();
            }
            return Some(inner);
        }

        // 负号 + 数字
        if c == '-' {
            let c2 = self.peek2();
            if c2.is_some_and(|c| c.is_ascii_digit() || c == '.') {
                self.advance(); // 消费 '-'
                return self.parse_number().map(|n| match n {
                    CalcNode::Number(v, u) => CalcNode::Number(-v, u),
                    other => other,
                });
            }
            // CSS 自定义属性 --name
            if c2.is_some_and(|c| c == '-') {
                return self.parse_ident_or_func();
            }
        }

        // 数字
        if c.is_ascii_digit() || c == '.' {
            return self.parse_number();
        }

        // 标识符（函数名或常量）
        if c.is_ascii_alphabetic() || c == '_' {
            return self.parse_ident_or_func();
        }

        None
    }

    /// 解析数字 + 可选单位——peek+next 循环。
    fn parse_number(&mut self) -> Option<CalcNode> {
        let mut num_str = String::new();
        loop {
            match self.peek() {
                Some(c) if c.is_ascii_digit()
                    || c == '.'
                    || c == 'e'
                    || c == 'E'
                    || (c == '-' && (num_str.ends_with('e') || num_str.ends_with('E')))
                    || (c == '+' && (num_str.ends_with('e') || num_str.ends_with('E'))) =>
                {
                    num_str.push(c);
                    self.advance();
                }
                _ => break,
            }
        }
        let n = num_str.parse::<f64>().ok()?;

        // 单位
        let mut unit = String::new();
        while self.peek().is_some_and(|c| is_unit_char(c)) {
            unit.push(self.advance().unwrap());
        }
        let unit = if unit.is_empty() { None } else { Some(unit) };
        Some(CalcNode::Number(n, unit))
    }

    /// 解析标识符或函数调用。
    fn parse_ident_or_func(&mut self) -> Option<CalcNode> {
        let mut name = String::new();
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            name.push(self.advance().unwrap());
        }
        self.skip_ws();

        // 函数调用
        if self.peek().is_some_and(|c| c == '(') {
            self.advance();
            self.parse_func_args(&name)
        } else {
            // 常量
            match name.to_lowercase().as_str() {
                "pi" => Some(CalcNode::Number(std::f64::consts::PI, None)),
                "e" => Some(CalcNode::Number(std::f64::consts::E, None)),
                _ => None,
            }
        }
    }

    /// 解析函数参数——var() 特殊处理。
    fn parse_func_args(&mut self, name: &str) -> Option<CalcNode> {
        if name == "var" {
            self.parse_var_args()
        } else {
            self.parse_generic_func_args(name)
        }
    }

    /// var() 参数解析——peek+next 收集名字。
    fn parse_var_args(&mut self) -> Option<CalcNode> {
        self.skip_ws();
        let mut var_name = String::new();
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            var_name.push(self.advance().unwrap());
        }
        self.skip_ws();

        let fallback = if self.peek().is_some_and(|c| c == ',') {
            self.advance();
            self.skip_ws();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.skip_ws();
        if self.peek().is_some_and(|c| c == ')') {
            self.advance();
        }
        if var_name.is_empty() {
            None
        } else {
            Some(CalcNode::Var { name: var_name, fallback })
        }
    }

    /// 通用函数参数解析。
    fn parse_generic_func_args(&mut self, name: &str) -> Option<CalcNode> {
        let mut args: Vec<CalcNode> = Vec::new();
        loop {
            self.skip_ws();
            if !self.has_more() {
                return None;
            }
            if self.peek().is_some_and(|c| c == ')') {
                self.advance();
                break;
            }
            args.push(self.parse_expr()?);
            self.skip_ws();
            if self.peek().is_some_and(|c| c == ',') {
                self.advance();
            }
        }
        Some(CalcNode::Func {
            name: name.to_string(),
            args,
        })
    }
}

/// 判断字符是否可以作为单位的一部分。
fn is_unit_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '%'
}

// ─── 公开 API：消费 &str → 返回 Option<CalcNode> ──────────────────

/// 将 calc 表达式字符串解析为 `CalcNode`。
///
/// 支持：运算符优先级（`*` `/` 高于 `+` `-`）、括号、CSS 函数、var()。
/// 降级策略：解析失败时返回 None，调用方可回退到字符串处理。
#[tracing::instrument(level = "debug", fields(input = %input))]
pub fn parse_calc_expr(input: &str) -> Option<CalcNode> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let mut parser = Parser::new(input);
    let result = parser.parse_expr()?;
    parser.skip_ws();
    if parser.has_more() {
        return None;
    }
    Some(result)
}
