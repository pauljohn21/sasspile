# 词法分析器（待开发）

## 职责

将 SCSS/Sass 源代码转换为 Token 序列，支持双语法（SCSS 和缩进语法）。

## 计划文件结构

```
lexer/
├── mod.rs          # Lexer 入口
├── token.rs        # Token 枚举定义
├── lexer.rs        # 主词法分析逻辑
└── sass_syntax.rs  # .sass 缩进语法支持
```

## Token 类型

```rust
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

pub enum TokenKind {
    // 字面量
    Ident(String),
    Number(f64),
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
    AtKeyword,        // @use, @mixin 等
    Hash,             // #（用于 ID 选择器）
    Ampersand,        // &（父选择器）
    Pipe,             // | (选择器组合)
    
    // Sass 特有
    Indent,           // .sass 缩进
    Dedent,           // .sass 缩出
    
    // 其他
    Whitespace,
    Comment(LineComment | BlockComment),
    Eof,
}
```

## 插值处理

`#{}` 是关键难点，需要在以下上下文中正确识别：
- 选择器中
- 属性名中
- 属性值中
- 字符串内部

## 测试重点

- 基本 token 识别
- 数字（含科学计数法、单位）
- 字符串（单引号、双引号、转义）
- 插值边界识别
- 缩进语法 Indent/Dent 序列
