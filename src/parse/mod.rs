//! —— 语法分析器 ——
//!
//! ParseStream 实现 `Iterator<Item = Result<Node>>`，消费 token 流产出 AST 节点。
//!
//! ```text
//! ParseStream::new(tokens)
//!     │
//!     .next()  →  Option<Result<Node>>
//!     │              ├── None = 流结束
//!     │              └── Some(Result<Node>)
//!
//! .collect::<Result<Vec<_>>>()
//!     │
//!     Vec<Node> → Ast { nodes }
//! ```

pub mod ast;
mod ast_impl;
pub mod at_rule_kinds;

use crate::error::Result;
use crate::lex::token::Token;
use ast::*;

/// 解析产物——统一 SCSS。
pub type Parsed = Ast;

// ═══════════════════════════════════════════════════════════════════════════
// ParseStream —— Iterator 实现
// ═══════════════════════════════════════════════════════════════════════════

/// 节点迭代器——消费 token 流产出 Node 的 Iterator。
///
/// 内部位置推进自然发生，外部只需 `next()` 驱动。
pub struct ParseStream<'tok> {
    tokens: &'tok [Token],
    pos: usize,
    /// 是否在规则体内（用于 @forward/@use 验证）。
    in_body: bool,
    /// 是否已解析过非模块规则。
    saw_other_rule: bool,
}

impl<'tok> ParseStream<'tok> {
    /// 创建新的 ParseStream。
    pub fn new(tokens: &'tok [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            in_body: false,
            saw_other_rule: false,
        }
    }

    // ── 基础流操作 ──────────────────────────────────────────────────────

    /// 查看当前 token（不消费）。
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// 查看第 n 个后续 token（不消费）。
    fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n)
    }

    /// 消费当前 token，pos 推进。
    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        if !matches!(t, Some(Token::Eof) | None) {
            self.pos += 1;
        }
        t
    }

    /// 是否在末尾（跳过 whitespace 后检查）。
    fn at_end(&self) -> bool {
        let mut i = self.pos;
        while matches!(self.tokens.get(i), Some(Token::Whitespace)) {
            i += 1;
        }
        matches!(self.tokens.get(i), None | Some(Token::Eof))
    }

    /// 跳过 whitespace 和行内注释。
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(Token::Whitespace) | Some(Token::Comment(_, true))) {
            self.pos += 1;
        }
    }

    /// 跳过所有 whitespace 和注释。
    fn skip_ws_and_comments(&mut self) {
        while matches!(self.peek(), Some(Token::Whitespace) | Some(Token::Comment(_, _))) {
            self.pos += 1;
        }
    }

    /// 消费期望 token。
    fn expect(&mut self, tok: &Token) -> Result<()> {
        self.skip_ws();
        if self.peek() == Some(tok) {
            self.advance();
            Ok(())
        } else {
            Err(crate::error::SassError::Parse {
                expected: tok.to_string(),
                found: self.peek().map_or("EOF".into(), |t| t.to_string()),
            })
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Iterator trait 实现
// ═══════════════════════════════════════════════════════════════════════════

impl<'tok> Iterator for ParseStream<'tok> {
    type Item = Result<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();
        if self.at_end() {
            return None;
        }
        Some(self.parse_node())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 公开入口
// ═══════════════════════════════════════════════════════════════════════════

/// 语法分析器命名空间。
pub struct Parser;

impl Parser {
    /// 解析入口——消费 token 流，产出 AST。
    pub fn parse(tokens: &[Token]) -> Result<Ast> {
        ParseStream::new(tokens)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map(|nodes| Ast { nodes })
    }
}

/// 顶层 parse 函数。
pub fn parse(tokens: &[Token]) -> Result<Ast> {
    Parser::parse(tokens)
}

// ═══════════════════════════════════════════════════════════════════════════
// 模块声明
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) mod at_rules;
pub(crate) mod at_rules_flow;
pub(crate) mod at_rules_modules;
pub(crate) mod expr;
pub(crate) mod nodes;
pub(crate) mod params;
