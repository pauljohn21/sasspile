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
    pub fn parse_value(&mut self) -> Result<Value> {
        let first = self.parse_expr(0)?;
        self.skip_ws();
        // 逗号分隔列表
        if self.peek() == Some(&Token::Comma) {
            let mut items = vec![first];
            while self.peek() == Some(&Token::Comma) {
                self.advance();
                self.skip_ws();
                items.push(self.parse_expr(0)?);
                self.skip_ws();
            }
            return Ok(Value::List(items, Separator::Comma, false));
        }
        Ok(first)
    }

    pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<Value> {
        self.skip_ws();
        let mut lhs = self.parse_prefix()?;
        loop {
            self.skip_ws();
            let (op, bp) = match self.peek_binding_power() {
                Some(v) => v,
                None => {
                    // 空格分隔列表——仅顶层（min_bp=0）
                    if min_bp == 0 && self.is_value_start() {
                        let mut items = vec![lhs.clone()];
                        loop {
                            self.skip_ws();
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
