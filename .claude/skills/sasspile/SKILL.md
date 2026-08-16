---
name: sasspile
description: "Sasspile SCSS 编译器开发技能。触发：实现编译器新模块、添加 sass-spec 测试、调试管道阶段、检查文件行数。使用场景：Phase 1-12 任务实施、模块拆分、sass-spec 集成、OpenSpec 工作流。"
---

# sasspile — SCSS 编译器开发技能

## 技能概述

本技能提供 sasslipe SCSS 编译器开发的标准化工作流程，确保：
- 单文件 ≤ 400 行
- 仅用 tracing 宏（零 println!）
- 测试与 src/ 分离
- 模块化、可独立的开发单元

## 触发条件

- 用户提及"实现 lexer/parser/value/css/pipeline"
- 用户提及"sass-spec"、"测试"、"用例"
- 用户提及"拆分文件"、"文件太长"
- 用户提及"Phase"、"任务"
- 用户需要调试编译管道

## 工作流程

### A. 实现新模块

```bash
# 1. 读取相关 spec
cat openspec/changes/scss-compiler/specs/{module}/spec.md

# 2. 创建模块文件
mkdir -p sasspile/src/{module}

# 4. 实现（严守行数限制）
# - mod.rs: 仅 re-export（≤ 100 行）
# - 核心类型定义
# - 操作实现拆分到独立文件

# 5. 添加到 lib.rs
# pub mod {module};

# 6. 构建验证
cargo build -p sasspile
```

### B. 文件拆分（当接近 400 行）

1. 识别职责边界
2. 拆分到 `*_ops.rs` / `*_ser.rs` / `*_ext.rs`
3. 更新 mod.rs re-export
4. 确保所有测试仍通过

### C. OpenSpec 任务实施

```bash
# 查看任务
openspec instructions apply --change "scss-compiler" --json

# 实施单个任务
# 1. 读取相关 spec
# 2. 实现代码
# 3. 添加测试到 tests/
# 4. 标记任务完成（tasks.md 中 - [ ] → - [x]）
```

### D. 运行 sass-spec

```bash
# 待实现
cargo test -p sasspile --test sass_spec
```

## 模块模板

### source/span.rs 模板

```rust
//! Source span definitions.

use std::ops::Range;

/// Byte range in source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub start: u32,
    pub end: u32,
}

impl SourceSpan {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        (self.end - self.start) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl From<Range<usize>> for SourceSpan {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start as u32,
            end: range.end as u32,
        }
    }
}

impl From<SourceSpan> for Range<usize> {
    fn from(span: SourceSpan) -> Self {
        span.start as usize..span.end as usize
    }
}
```

### value/mod.rs 模板

```rust
//! Sass value types.
//!
//! All values are immutable and shareable across Tokio tasks via Arc.

use std::sync::Arc;

mod color;
mod coerce;
mod number;
mod ops;
mod ser;

pub use color::SassColor;
pub use number::{Number, Unit};

/// Sass 值类型.
#[derive(Debug, Clone)]
pub enum Value {
    /// Numeric value with optional unit.
    Number(Number),
    /// String (quoted or unquoted).
    String(String, Quoted),
    /// Boolean value.
    Boolean(bool),
    /// Sass null.
    Null,
    /// sRGB color.
    Color(SassColor),
    /// List with separator.
    List(Vec<Value>, Separator),
    /// Key-value map.
    Map(Vec<(Value, Value)>),
    /// Argument list (trailing kwargs).
    ArgList(Vec<Value>),
    /// Function reference.
    Function(String),
    /// Calc() expression.
    Calculation(String),
    /// Error sentinel.
    Error(String),
}

/// Quoted vs unquoted string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quoted {
    Quoted,
    Unquoted,
}

/// List separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separator {
    Comma,
    Space,
    Slash,
    Undecided,
}
```

### lexer/token.rs 模板

```rust
//! Token types for SCSS/Sass lexing.

use crate::source::SourceSpan;

/// Token with source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

/// Token kind enumeration.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Literals ─────────────────────────────
    Ident(String),
    Number(f64, Option<String>),  // value + unit
    String(String),
    Url(String),
    Color(u32),       // #rrggbb

    // ── Operators ────────────────────────────
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,               // ==
    NotEq,            // !=
    Greater,          // >
    Less,             // <
    GreaterEq,        // >=
    LessEq,           <==
    And,              // and
    Or,               // or
    Not,              // not

    // ── Delimiters ───────────────────────────
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    DotDotDot,        // ...

    // ── Special ──────────────────────────────
    Interpolation,    // #{
    AtKeyword(String), // @use, @mixin, etc.
    Hash,             // # (for ID)
    Ampersand,        // & (parent selector)
    Dollar,           // $ (variable prefix)
    Variable(String), // $var

    // ── Sass-specific ────────────────────────
    Indent,           // .sass
    Dedent,           // .sass

    // ── Other ────────────────────────────────
    Whitespace,
    Eof,
}
```

### 测试模板 (tests/{module}_spec.rs)

```rust
//! Tests for {module}.

use sasspile::value::{Value, Number, Unit};

#[test]
fn test_basic_creation() {
    // Arrange
    let value = Value::Number(Number::new(16.0, Unit::Px));

    // Act
    let css = value.to_css_string();

    // Assert
    assert_eq!(css, "16px");
}

#[test]
fn test_equality() {
    let a = Value::Number(Number::new(1.0, Unit::None));
    let b = Value::Number(Number::new(1.0, Unit::None));
    assert_eq!(a, b);
}

#[test]
fn test_serialization_roundtrip() {
    // TODO: Implement roundtrip test
}
```

## 实施检查清单

```bash
# 验证单文件行数
find sasspile/src -name "*.rs" -exec wc -l {} \; | sort -rn | head -20

# 无 println!/eprintln!
grep -rn "println!\|eprintln!" sasspile/src/ && echo "FOUND!" || echo "Clean"

# 无内联 #[cfg(test)]
grep -rn "#\[cfg(test)\]" sasspile/src/ && echo "FOUND!" || echo "Clean"

# 所有测试在 tests/
ls sasspile/tests/

# cargo clippy 无 warning
cargo clippy -p sasspile -- -D warnings
```

## Phase 清单

| Phase | 模块 | 关键行数上限 |
|-------|------|-------------|
| 1 | source/, value/, diagnostics/ | 400 |
| 2 | lexer/ | 400 |
| 3 | parser/ | 400 |
| 4 | semantic/ | 400 |
| 5 | eval/ | 400 |
| 6 | builtin/ | 400 |
| 7 | css/ | 400 |
| 8 | incremental/ | 400 |
| 9 | pipeline/ | 400 |

## 常用命令速查

```bash
# 构建
cargo build -p sasspile

# 运行 CLI（待实现）
cargo run -p sasspile -- input.scss -o output.css

# 测试单个模块
cargo test -p sasspile --test lexer_spec

# Watch 模式（待实现）
cargo run -p sasspile -- input.scss --watch

# 完整验证
cargo build -p sasspile && \
cargo test -p sasspile && \
cargo clippy -p sasspile -- -D warnings
```

## 错误类型约定

```rust
//! New error variant format:
#[error("invalid syntax at {span:?}: {message}")]
InvalidSyntax { span: SourceSpan, message: String },

#[error("incompatible units: {0} and {1}")]
IncompatibleUnits(String, String),
```

## 待办事项

1. 完善 sass-spec 测试运行器
2. 实现 source map v3 输出
3. 集成 fsnotify 用于 watch 模式
