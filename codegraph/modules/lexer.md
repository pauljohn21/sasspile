# 词法分析器 ✅ 已完成

## 职责

将 SCSS/Sass 源代码转换为 Token 序列，支持双语法（SCSS 和缩进语法）。

## 文件结构（实际）

```
lexer/
├── mod.rs          # Lexer 入口 + tokenize 便捷函数
├── token.rs        # Token/TokenKind 定义
├── lex.rs          # 主词法分析逻辑
└── sass_syntax.rs  # .sass 缩进语法支持
```

## Token 定义

**文件: `sasspile/src/lexer/token.rs`**

```rust
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

pub enum TokenKind {
    // 字面量
    Ident(String),
    Number { value: f64, raw: String },
    String(String),
    Url(String),
    Color(u32),       // #rgb / #rrggbb
    
    // 运算符
    Plus, Minus, Star, Slash, Percent,
    Eq, NotEq, Greater, Less, GreaterEq, LessEq,
    And, Or, Not,
    
    // 分隔符
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semicolon, Colon, Comma, Dot, DotDotDot,
    
    // 特殊
    Interpolation,    // #{
    AtKeyword(String), // @use, @mixin 等
    Hash,             // #（用于 ID 选择器）
    Ampersand,        // &（父选择器）
    Pipe,             // | (选择器组合)
    
    // 其他
    Whitespace,
    Comment(LineComment | BlockComment),
    Eof,
}
```

## 使用方式

```rust
use sasspile::lexer::{Lexer, tokenize};

// 方式一：便捷函数
let (tokens, diagnostics) = tokenize(source);

// 方式二：构建 Lexer
let lexer = Lexer::new(source);
let (tokens, diag) = lexer.tokenize();
```

## 插值处理

`#{}` 在以下上下文中正确识别：
- 选择器中
- 属性名中
- 属性值中
- 字符串内部

## 已修复问题

- ✅ 反斜杠转义处理
- ✅ 注释后中断 bug
- ✅ 缩进语法的 Indent/Dedent

## 测试

- `tests/lexer_spec.rs`
