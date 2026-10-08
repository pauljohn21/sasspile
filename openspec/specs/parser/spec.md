# Parser Specification

## Purpose

将 `Token` 流转换为 `Observable<SassAstNode>` 流式语法树节点序列，完整表达 SCSS 所有语法结构。由 `CompileBuilder::build` 内部调用，使用 `scan` 增量式解析构建 AstNode 流。

## Requirements

### Requirement: 增量式解析
Parser SHALL 使用 `scan(ParserState, |state, token| Observable<Vec<SassAstNode>>)` 增量接收 Token。每当 `ParserState` 积累足够 Token 以构成一个完整语句时，SHALL 产出对应的 `Vec<SassAstNode>` 并通过 `flat_map` 展平。

#### Scenario: Complete style declaration
- **WHEN** Parser 接收到 `Ident("color")`, `Colon`, `Ident("red")`, `Semicolon`
- **THEN** SHALL 产出 `SassAstNode::StyleDecl { property: "color", value: Literal(String("red")) }`

#### Scenario: Nested rule processing
- **WHEN** Parser 接收 `.a { .b { color: red; } }`
- **THEN** SHALL 产出 `SassAstNode::Rule` 包含子 `SassAstNode::Rule` 的嵌套结构

### Requirement: @-rules 解析
Parser SHALL 识别并解析所有 @-rule 类型：`@media`、`@supports`、`@if`、`@for`、`@each`、`@while`、`@mixin`、`@include`、`@function`、`@return`、`@use`、`@forward`、`@extend`、`@warn`、`@debug`。

#### Scenario: @media query parsing
- **WHEN** Parser 遇到 `@media screen and (min-width: 768px) { a { color: red; } }`
- **THEN** SHALL 产出 `SassAstNode::Media { query: "screen and (min-width: 768px)", inner: [StyleDecl] }`

#### Scenario: @for loop with inclusive range
- **WHEN** Parser 遇到 `@for $i from 1 through 3 { ... }`
- **THEN** SHALL 产出 `SassAstNode::For { var: "i", from: 1, to: 3, inclusive: true, body: [...] }`

#### Scenario: @each with list
- **WHEN** Parser 遇到 `@each $item in a, b, c { ... }`
- **THEN** SHALL 产出 `SassAstNode::Each { vars: ["item"], list: [...], body: [...] }`

### Requirement: 插值解析
Parser SHALL 处理属性名、属性值、选择器、@-rule query 中的 `#{expr}` 插值。插值内的表达式 SHALL 作为子 `AstNode` 解析，插值节点 SHALL 包含解析后的子节点。

#### Scenario: Interpolated selector
- **WHEN** Parser 遇到 `.#{$class}-suffix`
- **THEN** SHALL 产出 `SassAstNode::Interpolation` 内嵌 `AstNode::VariableRef { name: "$class" }`

#### Scenario: Interpolated property value
- **WHEN** Parser 遇到 `width: #{$size}px`
- **THEN** SHALL 产出包含 `Interpolation(VariableRef("$size"))` 和 `Literal("px")` 混合的复合值节点

### Requirement: 变量声明与引用
Parser SHALL 解析 `$name: value;` 为 `SassAstNode::VariableDecl { name, value, scope_id }`。变量引用 `$name` SHALL 解析为 `AstNode::VariableRef { name, scope_id }`。

#### Scenario: Variable declaration
- **WHEN** Parser 遇到 `$primary: #369;`
- **THEN** SHALL 产出 `SassAstNode::VariableDecl { name: "primary", value: AstNode::Literal(Color(51,102,153,255)), scope_id: <current> }`

#### Scenario: Variable reference in expression
- **WHEN** Parser 遇到 `color: $primary`
- **THEN** SHALL 产出 `AstNode::Literal(VariableRef { name: "primary", scope_id: <current> })`

### Requirement: 值类型解析
Parser SHALL 解析整数/浮点数（带单位）、字符串、颜色、布尔值、null、列表、Maps（key: value）、函数调用、算术/比较表达式。

#### Scenario: Number with unit
- **WHEN** Parser 遇到 `16px`
- **THEN** SHALL 产出 `AstNode::Literal(Number(16.0))` 带单位元数据

#### Scenario: Map literal
- **WHEN** Parser 遇到 `("a": 1, "b": 2)`
- **THEN** SHALL 产出 `AstNode::Literal(Map([("a", Number(1)), ("b", Number(2))]))`

### Requirement: 错误恢复
当 Parser 遇到无法识别的语法结构时，SHALL 跳过当前 Token 并尝试从下一个同步点继续。错误信息 SHALL 包含 `pos` 和上下文。

#### Scenario: Missing semicolon recovery
- **WHEN** Parser 遇到 `$x: 1 $y: 2` (缺 `;`)
- **THEN** SHALL 在 `$y` 前尝试同步并产出两个 VariableDecl

### Requirement: 未闭合分隔符检测
流结束后，Parser SHALL 检查 `ParserState` 中是否存在未闭合的 `{`、`(`、`[`。若无闭合 SHALL 产出对应错误。

#### Scenario: Unclosed brace
- **WHEN** Parser 处理 `.a { color: red;` (无 `}`)
- **THEN** SHALL 在流末尾产出 ParseError 指出未闭合的 `{`

### Requirement: Observable 生命周期
`create_ast_stream` 返回 `AstStream = SharedBoxedObservable<'static, AstNode, Infallible>`。下游取消订阅 SHALL 立即停止 Token 消费。

#### Scenario: Early completion
- **WHEN** AST 在被完全消费前订阅者取消
- **THEN** ParserState SHALL 立即完成 Token 处理
