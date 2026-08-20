//! Pratt 表达式解析 + 数值/颜色解析。
//!
//! 包含 parse_expr/is_value_start/parse_prefix/peek_binding_power 方法
//! 和 parse_number/parse_hash_color/hex2/hex1 自由函数。

mod prefix;

#[allow(unused_imports)]
pub(crate) use prefix::{parse_hash_color, parse_number};

use super::ast::*;
use super::Parser;
use crate::error::Result;
use crate::lex::token::Token;

impl<'tok> Parser<'tok> {
    // —— Pratt 表达式解析 ——
    /// 解析值表达式（顶层，到 ; 或 } 停止）。
    /// 用于变量赋值、函数参数等需要求值的上下文——`/` 做除法。
    pub fn parse_value(&mut self) -> Result<Value> {
        self.parse_value_with_slash(false)
    }

    /// 解析声明值表达式——`/` 作为斜杠分隔符保留。
    /// 用于 CSS 声明值（如 `a {b: 1/2}` → `1/2`）。
    pub fn parse_decl_value(&mut self) -> Result<Value> {
        self.parse_value_with_slash(true)
    }

    /// 解析值表达式核心逻辑。
    /// `slash_as_sep=true` 时，顶层 `/` 被视为斜杠分隔列表而非除法。
    fn parse_value_with_slash(&mut self, slash_as_sep: bool) -> Result<Value> {
        let first = self.parse_expr_slash(0, slash_as_sep)?;
        self.skip_ws();
        // 逗号分隔列表
        if self.peek() == Some(&Token::Comma) {
            let mut items = vec![first];
            while self.peek() == Some(&Token::Comma) {
                self.advance();
                self.skip_ws();
                items.push(self.parse_expr_slash(0, slash_as_sep)?);
                self.skip_ws();
            }
            return Ok(Value::List(items, Separator::Comma, false));
        }
        Ok(first)
    }

    pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<Value> {
        self.parse_expr_slash(min_bp, false)
    }

    /// Pratt 表达式解析。
    /// `slash_as_sep=true` 且 `min_bp==0` 时，`/` 构建斜杠分隔列表。
    fn parse_expr_slash(&mut self, min_bp: u8, slash_as_sep: bool) -> Result<Value> {
        self.skip_ws();
        let mut lhs = self.parse_prefix()?;
        loop {
            self.skip_ws();
            // 声明值上下文 + 顶层 + 遇到 Slash → 斜杠分隔列表
            // 但以下情况 / 做除法：
            // 1. / 后面的表达式包含 +、-、*、% 运算符
            // 2. / 前面是括号表达式 (如 (1)/2)
            // 3. / 前面是变量引用 (如 $x/2)
            let is_division_context = matches!(lhs, Value::Paren(_) | Value::Variable(_));
            if slash_as_sep && min_bp == 0 && matches!(self.peek(), Some(Token::Slash)) && !is_division_context {
                // 检查 / 后面是否有其他算术运算符（+、-、*、%）
                let has_outer_math = self.slash_followed_by_arith_op();
                if !has_outer_math {
                    let mut slash_items = vec![lhs.clone()];
                    while self.peek() == Some(&Token::Slash) {
                        self.advance(); // 消费 /
                        self.skip_ws();
                        // 解析下一个项（min_bp=6 确保不递归进入斜杠逻辑）
                        let item = self.parse_expr_slash(6, false)?;
                        slash_items.push(item);
                        self.skip_ws();
                    }
                    if slash_items.len() > 1 {
                        lhs = Value::List(slash_items, Separator::SlashLiteral, false);
                    }
                    // 斜杠列表后面可能有空格列表——继续循环
                    continue;
                }
                // 有外部算术运算符 → / 做除法，走正常 Pratt 路径
            }
            let (op, bp) = match self.peek_binding_power() {
                Some(v) => v,
                None => {
                    // 空格分隔列表——仅顶层（min_bp=0）
                    if min_bp == 0 && self.is_value_start() {
                        let mut items = vec![lhs.clone()];
                        loop {
                            self.skip_ws();
                            // 空格列表中的 / 始终作为斜杠分隔符保留
                            // 例如 1 2/3 4 → [1, 2/3, 4]
                            if matches!(self.peek(), Some(Token::Slash)) {
                                let last = items.pop().unwrap();
                                let mut slash_items = vec![last];
                                while self.peek() == Some(&Token::Slash) {
                                    self.advance();
                                    self.skip_ws();
                                    let item = self.parse_expr_slash(6, false)?;
                                    slash_items.push(item);
                                    self.skip_ws();
                                }
                                let slash_list = if slash_items.len() > 1 {
                                    Value::List(slash_items, Separator::SlashLiteral, false)
                                } else {
                                    slash_items.into_iter().next().unwrap()
                                };
                                items.push(slash_list);
                                continue;
                            }
                            // 先检查二元运算符——仅算术运算符（+,-,*,/,%）才在列表内消费
                            // 低优先级运算符（or,and,==,!=,<,>,<=,>=）不在此消费
                            if let Some((_, bp)) = self.peek_binding_power()
                                && bp >= 4
                            {
                                let last = items.pop().unwrap();
                                let binop_result = self.parse_expr_rest(last, 4)?;
                                items.push(binop_result);
                                continue;
                            }
                            if !self.is_value_start() {
                                break;
                            }
                            items.push(self.parse_prefix()?);
                        }
                        if items.len() > 1 {
                            lhs = Value::List(items, Separator::Space, false);
                            continue; // 继续检查后续运算符（如 / ）
                        }
                    }
                    break;
                }
            };
            if bp < min_bp {
                break;
            }
            self.advance(); // 消费运算符
            self.skip_ws();
            let rhs = self.parse_expr(bp + 1)?;
            lhs = Value::BinOp(Box::new(BinOp {
                op,
                left: lhs,
                right: rhs,
            }));
        }
        Ok(lhs)
    }

    /// 从已有的 lhs 继续解析二元运算符表达式（不构建空格列表）。
    /// 在空格列表内解析算术子表达式——`/` 始终做除法。
    fn parse_expr_rest(&mut self, mut lhs: Value, min_bp: u8) -> Result<Value> {
        loop {
            self.skip_ws();
            let (op, bp) = match self.peek_binding_power() {
                Some(v) => v,
                None => break,
            };
            if bp < min_bp {
                break;
            }
            self.advance();
            self.skip_ws();
            let rhs = self.parse_expr(bp + 1)?;
            lhs = Value::BinOp(Box::new(BinOp {
                op,
                left: lhs,
                right: rhs,
            }));
        }
        Ok(lhs)
    }

    /// 检查字符串是否为 SCSS 关键字（不应被拼接进字符串）。
    fn is_keyword(&self, s: &str) -> bool {
        matches!(
            s,
            "through" | "from" | "to" | "and" | "or" | "not" | "in" | "with"
                | "show" | "hide" | "as" | "using" | "else"
        )
    }

    /// 检查当前 token 是否是值起始 token（排除关键字）。
    pub(crate) fn is_value_start(&self) -> bool {
        if let Some(Token::Ident(s)) = self.peek()
            && matches!(
                s.as_str(),
                "through" | "from" | "to" | "and" | "or" | "not" | "in" | "with"
                    | "show" | "hide" | "as" | "using" | "else"
            )
        {
            return false;
        }
        matches!(
            self.peek(),
            Some(Token::Number(_))
                | Some(Token::String(_, _))
                | Some(Token::Ident(_))
                | Some(Token::Hash(_))
                | Some(Token::Dollar(_))
                | Some(Token::Interp(_))
                | Some(Token::LParen)
                | Some(Token::LBracket)
                | Some(Token::Minus)
                | Some(Token::Percent)
                | Some(Token::True)
                | Some(Token::False)
                | Some(Token::Null)
        )
    }

    /// 检查当前位置的 / 后面是否有 +、-、*、% 算术运算符。
    /// 用于区分 `1/2`（斜杠分隔符）和 `1/2+1`（/ 做除法）。
    /// 跳过 / 后面的数字字面量和连续的 / 数字，检查是否遇到 +、-、*、%。
    fn slash_followed_by_arith_op(&self) -> bool {
        let mut i = self.pos + 1; // 跳过当前的 Slash
        // 跳过空白
        while i < self.tokens.len() && matches!(self.tokens[i], Token::Whitespace) {
            i += 1;
        }
        // 遍历直到找到 +、-、*、% 或遇到 ;、}、) 等终止符
        let mut paren_depth = 0i32;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::Whitespace => { i += 1; }
                Token::Number(_) => { i += 1; }
                Token::Slash => { i += 1; }
                Token::Percent => { i += 1; }
                // 一元负号（space-before + no-space-after + Number）不算算术运算符
                Token::Minus => {
                    let has_ws_before = i > 0
                        && matches!(self.tokens.get(i - 1), Some(Token::Whitespace));
                    let next = self.tokens.get(i + 1);
                    let has_ws_after = matches!(next, Some(Token::Whitespace) | None);
                    if has_ws_before && !has_ws_after && matches!(next, Some(Token::Number(_))) {
                        // 一元负号，继续
                        i += 1;
                    } else {
                        // 二元减法 → 有外部算术运算
                        return true;
                    }
                }
                Token::Plus => return true,
                Token::Star => return true,
                Token::LParen => { paren_depth += 1; i += 1; }
                Token::RParen => {
                    if paren_depth == 0 { break; }
                    paren_depth -= 1;
                    i += 1;
                }
                Token::Semicolon | Token::RBrace | Token::Comma | Token::RBracket
                | Token::Eof | Token::Ident(_) | Token::Dollar(_) | Token::Hash(_)
                | Token::Interp(_) | Token::LBracket | Token::True | Token::False
                | Token::Null => break,
                _ => { i += 1; }
            }
        }
        false
    }

    pub(crate) fn peek_binding_power(&self) -> Option<(BinOpKind, u8)> {
        match self.peek() {
            Some(Token::Or) => Some((BinOpKind::Or, 1)),
            Some(Token::And) => Some((BinOpKind::And, 2)),
            Some(Token::Eq) => Some((BinOpKind::Eq, 3)),
            Some(Token::NotEq) => Some((BinOpKind::NotEq, 3)),
            Some(Token::Less) => Some((BinOpKind::Lt, 3)),
            Some(Token::Greater) => Some((BinOpKind::Gt, 3)),
            Some(Token::LessEq) => Some((BinOpKind::LtEq, 3)),
            Some(Token::GreaterEq) => Some((BinOpKind::GtEq, 3)),
            Some(Token::Plus) => Some((BinOpKind::Add, 4)),
            Some(Token::Minus) => {
                // Sass 规则：space-before + no-space-after + Number = 一元负号
                let has_ws_before = self.pos > 0
                    && matches!(self.tokens.get(self.pos - 1), Some(Token::Whitespace));
                let next = self.tokens.get(self.pos + 1);
                let has_ws_after = matches!(next, Some(Token::Whitespace) | None);
                if has_ws_before && !has_ws_after && matches!(next, Some(Token::Number(_))) {
                    None // 一元负号，不是二元运算符
                } else {
                    Some((BinOpKind::Sub, 4))
                }
            }
            Some(Token::Star) => Some((BinOpKind::Mul, 5)),
            Some(Token::Slash) => Some((BinOpKind::Div, 5)),
            Some(Token::Percent) => Some((BinOpKind::Mod, 5)),
            _ => None,
        }
    }
}
