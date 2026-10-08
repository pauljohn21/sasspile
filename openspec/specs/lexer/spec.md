# Lexer Specification

## Purpose

将 SCSS/SASS/CSS 源代码转换为 `Shared Observable<Token>` 流式词法单元序列，作为编译管线的第一个阶段。由 `CompileBuilder::build` 内部调用，支持三种语法模式切换。

## Requirements

### Requirement: Token 类型完整覆盖
Lexer 定义的 `Token` enum SHALL 涵盖 Sass 规范中所有词法单元：标识符、数字、字符串、`#{}`插值、操作符(`==`、`!=`、`<=`、`>=`、`+`、`-`、`*`、`/`、`%`)、括号、分号、冒号、逗号、点、`@`-rules(`@media`、`@supports`、`@if`、`@for`、`@each`、`@while`、`@mixin`、`@include`、`@function`、`@return`、`@use`、`@forward`、`@extend`、`@warn`、`@debug`)、`&`父选择器、`$`变量前缀以及 `Eof` 结束标记。

#### Scenario: SCSS source with all token types
- **WHEN** Lexer 处理 `"$color: #369; .a { &:hover { color: $color; } }"`
- **THEN** Token 流 SHALL 依次包含 `Dollar`, `Ident("color")`, `Colon`, `Hash`, `Ident("369")`, `Semicolon`, `Dot`, `Ident("a")`, `LBrace`, `Ampersand`, `Colon`, `Ident("hover")`, `LBrace`, `Ident("color")`, `Colon`, `Dollar`, `Ident("color")`, `Semicolon`, `RBrace`, `RBrace`, `Eof`

#### Scenario: At-rule recognition
- **WHEN** Lexer 遇到 `@media`
- **THEN** SHALL 输出 `Token::AtMedia` 而非 `Token::At` + `Token::Ident("media")` 的组合

#### Scenario: Interpolation sequence
- **WHEN** Lexer 遇到 `#{`
- **THEN** SHALL 输出 `Token::InterpStart` 单一词法单元

### Requirement: LexerState 扫描式分析
Lexer 内部 SHALL 使用 `scan(LexerState, |state, ch| { ... })` 响应式算子逐字符扫描。`LexerState` 维护当前扫描位置、字符串上下文、插值深度等可变状态。每个字符消费后 SHALL 产出零个或多个 Token（`Vec<Token>`），通过 `flatten()` 展平到下游。

#### Scenario: String token with escaped characters
- **WHEN** Lexer 处理 `"a\"b"` 字符串
- **THEN** SHALL 输出 `Token::Str("a\"b")` 将 `\"` 视为转义而非字符串终止

#### Scenario: Multiline input processing
- **WHEN** Lexer 处理包含 `\n` 的源码
- **THEN** 每行末的 `;` 可省略时 SHALL 以换行为隐式分号边界

### Requirement: 语法模式选择
`CompileBuilder::build` 内部的 lexer 阶段 SHALL 根据 `syntax` 参数（`Scss` / `Sass` / `Css`，通过 `.syntax()` 配置）初始化不同的 `LexerState`。SCSS 模式 SHALL 支持 `{}` 块语法；Sass 模式 SHALL 支持缩进语法；CSS 模式 SHALL 跳过所有 Sass 专有特性（变量、嵌套、混入等）。

#### Scenario: SCSS mode scanning
- **WHEN** `create_token_stream("$x: 1;", InputSyntax::Scss)` 被调用
- **THEN** SHALL 识别 `$x` 为 Dollar + Ident 变量声明语法

#### Scenario: CSS mode ignores Sass variables
- **WHEN** `create_token_stream("$color: red;", InputSyntax::Css)` 被调用
- **THEN** SHALL 将 `$color` 视为普通标识符文本而非变量声明

### Requirement: 插值和字符串上下文
Lexer SHALL 跟踪以下上下文状态：(1) 是否处于双引号字符串内；(2) 是否处于单引号字符串内；(3) 插值 `#{}` 嵌套深度；(4) 是否处于注释内（`//` 单行注释、`/* */` 多行注释）。字符串内的字符 SHALL 不触发任何操作符或@-rule识别。

#### Scenario: Comment skipping
- **WHEN** Lexer 遇到 `// This is a comment\n`
- **THEN** SHALL 丢弃注释内容直至行尾，不出现在 Token 流中

#### Scenario: Interpolation inside string
- **WHEN** Lexer 遇到 `"hello #{$name}"`
- **THEN** SHALL 输出 `Token::Str(...)`, `Token::InterpStart`, `Token::Dollar`, `Token::Ident("name")`, `Token::RBrace`, `Token::Str(...)` 混合序列

### Requirement: Observable 生命周期管理
`create_token_stream` 返回的 `TokenStream` SHALL 是 `Shared + Send + Sync` 类型别名（`SharedBoxedObservable<'static, Token, Infallible>`）。订阅时 SHALL 触发扫描；取消订阅时 SHALL 停止扫描，不产生后续 Token。

#### Scenario: Early unsubscription
- **WHEN** 下游订阅者在前 5 个 Token 后取消订阅
- **THEN** Lexer SHALL 停止处理剩余源码，不产生后续 Token

### Requirement: Token 位置信息
每个 Token SHALL 携带源码位置信息（`pos: u32` 字节偏移），用于错误报告。位置 SHALL 在 `scan` 闭包内通过 `LexerState` 递增追踪。

#### Scenario: Error location tracking
- **WHEN** Parser 在第 42 字节处发现语法错误
- **THEN** 错误信息 SHALL 能引用 `Token.pos()` 获取第 42 字节的位置
