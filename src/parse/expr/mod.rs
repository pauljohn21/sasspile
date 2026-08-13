//! Pratt 表达式解析 + 数值/颜色解析。
//!
//! 包含 parse_expr/is_value_start/parse_prefix/peek_binding_power 方法
//! 和 parse_number/parse_hash_color/hex2/hex1 自由函数。

mod prefix;

#[allow(unused_imports)]
pub(crate) use prefix::{parse_hash_color, parse_number};

use super::ast::*;
use super::Parser;
use crate::error::{Result, SassError};
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
            // 检查 and/or 后面的 token（跳过空白）
            let mut after_op_idx = self.pos + 1;
            while matches!(self.tokens.get(after_op_idx), Some(Token::Whitespace)) {
                after_op_idx += 1;
            }
            let after_op = self.tokens.get(after_op_idx);
            // 检查 and/or 后面是否紧跟 (（无空白）→ 报错
            // 例如 `and(css(2))` 报错，`and (css(2))` 合法
            // 判断是否有空白：如果 after_op_idx == self.pos + 1，则没有空白
            if matches!(after_op, Some(Token::LParen))
                && matches!(op, BinOpKind::And | BinOpKind::Or)
                && after_op_idx == self.pos + 1
            {
                return Err(SassError::Parse {
                    expected: format!("Whitespace is required after {}", if matches!(op, BinOpKind::And) { "and" } else { "or" }),
                    found: "(".into(),
                });
            }
            // `and`/`or` 后面不能直接跟 `not`（需要括号）
            // 例如 `css(1) and not css(2)` 报错
            if matches!(after_op, Some(Token::Not))
                && matches!(op, BinOpKind::And | BinOpKind::Or)
            {
                return Err(SassError::Parse {
                    expected: "(".into(),
                    found: "not".into(),
                });
            }
            self.advance(); // 消费运算符
            self.skip_ws();
            let rhs = self.parse_expr(bp + 1)?;
            // 检查：and/or 的左操作数不能是 not
            // 例如 `not css(1) and css(2)` 报错
            if matches!(&lhs, Value::UnaryOp(UnaryOp::Not, _))
                && matches!(op, BinOpKind::And | BinOpKind::Or)
            {
                return Err(SassError::Parse {
                    expected: ":".into(),
                    found: if matches!(op, BinOpKind::And) { "and".into() } else { "or".into() },
                });
            }
            // 检查 and/or 混合：or 的右操作数不能是 BinOp(And, ...)，反之亦然
            // 例如 `css(1) or css(2) and css(3)` 报错
            // 注意：由于 and 的绑定优先级高于 or，`or css(2) and css(3)` 会被解析为
            // `or (and css(2) css(3))`，所以需要检查右操作数
            match (&op, &rhs) {
                (BinOpKind::Or, Value::BinOp(b)) if matches!(b.op, BinOpKind::And) => {
                    return Err(SassError::Parse {
                        expected: ":".into(),
                        found: "and".into(),
                    });
                }
                (BinOpKind::And, Value::BinOp(b)) if matches!(b.op, BinOpKind::Or) => {
                    return Err(SassError::Parse {
                        expected: ":".into(),
                        found: "or".into(),
                    });
                }
                _ => {}
            }
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
            // 检查 and/or 后面的 token（跳过空白）
            let mut after_op_idx = self.pos + 1;
            while matches!(self.tokens.get(after_op_idx), Some(Token::Whitespace)) {
                after_op_idx += 1;
            }
            let after_op = self.tokens.get(after_op_idx);
            // 检查 and/or 后面是否紧跟 (（无空白）→ 报错
            if matches!(after_op, Some(Token::LParen))
                && matches!(op, BinOpKind::And | BinOpKind::Or)
                && after_op_idx == self.pos + 1
            {
                return Err(SassError::Parse {
                    expected: format!("Whitespace is required after {}", if matches!(op, BinOpKind::And) { "and" } else { "or" }),
                    found: "(".into(),
                });
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
