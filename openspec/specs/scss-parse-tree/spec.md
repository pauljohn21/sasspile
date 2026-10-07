# SCSS Parse Tree Specification

## Purpose

定义独立的 Sass 语法解析树 `SassAstNode`，完整表达 Sass SCSS/SASS/CSS 语法，包括插值、父选择器、Maps、Lists 等 `AstNode` 无法承载的语法构造。作为 Parser 的直接产物和 lowering 转换层的输入。

## Requirements

### Requirement: 系统 SHALL 提供独立的 SassAstNode 解析树类型
系统 SHALL 定义 `pub enum SassAstNode`，用于表示 Parser 产出的完整 Sass 语法。SassAstNode SHALL 包含 AstNode 的所有变体，并额外支持插值表达式、父选择器、Map 字面量、List 字面量、`!default` 标记、注释节点。

#### Scenario: SassAstNode 涵盖完整语法
- **WHEN** Parser 遇到 `.btn { color: #{$prefix}-primary; }`
- **THEN** 产出包含 `SassAstNode::Interpolated` 变体的解析树，而非尝试将其纳入现有 AstNode

### Requirement: Parser SHALL 支持 SCSS、SASS、CSS 三种输入语法
系统 SHALL 提供三种 Parser 模式：ScssParser（`$` 变量、`@` 指令）、SassParser（缩进语法、无分号）、CssParser（纯 CSS，仅做 Pass-through）。Parser SHALL 通过 `InputSyntax` 枚举选择。

#### Scenario: SCSS 模式解析变量声明
- **WHEN** 输入为 `$color: red;`
- **THEN** ScssParser 产出 `SassAstNode::VariableDecl { name: "$color", value: ..., has_default: false }`

#### Scenario: Sass 模式解析缩进语法
- **WHEN** 输入为 `.btn\n  color: red`
- **THEN** SassParser 根据缩进层级正确解析嵌套关系

### Requirement: Parser SHALL 处理父选择器与插值
SassAstNode SHALL 包含 `ParentSelector` 和 `Interpolated(String)` 变体，允许选择器与属性值中出现 `&` 和 `#{}` 表达式。

#### Scenario: 父选择器在嵌套规则中使用
- **WHEN** 输入为 `a { &:hover { color: red; } }`
- **THEN** Parser 产出 `SassAstNode::Rule { selector: "a", inner: [SassAstNode::Rule { selector: "&:hover", ... }] }`

#### Scenario: 插值在选择器中使用
- **WHEN** 输入为 `.#{$class-name} { color: red; }`
- **THEN** Parser 产出选择器包含 `SassAstNode::Interpolated("$class-name")` 的规则节点

### Requirement: Parser SHALL 处理 Map 和 List 字面量
系统 SHALL 支持 Sass Map `("key": value, ...)` 和 List `(item, item, ...)` 字面量。SassAstNode SHALL 包含 `MapLiteral(Vec<(SassAstNode, SassAstNode)>)` 和 `ListLiteral(Vec<SassAstNode>)` 变体。

#### Scenario: Map 字面量解析
- **WHEN** 输入为 `$colors: (blue: #0d6efd, red: #dc3545);`
- **THEN** Parser 产出包含 `SassAstNode::MapLiteral` 的变量声明

#### Scenario: List 字面量解析
- **WHEN** 输入为 `$list: (a, b, c);`
- **THEN** Parser 产出包含 `SassAstNode::ListLiteral` 的变量声明

### Requirement: Parser SHALL 支持 `!default` 标记
变量声明节点 SHALL 包含 `has_default: bool` 字段。输入 `$color: red !default;` 时该字段为 `true`。

#### Scenario: 带 !default 的变量声明
- **WHEN** 输入为 `$color: red !default;`
- **THEN** 产出的变量声明中 `has_default == true`

### Requirement: Parser SHALL 通过 Observable 错误通道报告语法错误
当 Parser 在流结束后检测到未闭合分隔符时，SHALL 通过 Observable 的 `on_error` 通道发送 `crate::Error`，而非 panic 或静默忽略。Parser SHALL 使用 `check_unclosed_delimiters` 扫描 ParserState 中所有 token 来检测未闭合的 `(` 或 `{`。

#### Scenario: 未闭合的花括号
- **WHEN** 输入为 `.a { color: red`（缺少 `}`）
- **THEN** Parser 通过 `check_unclosed_delimiters` 扫描发现未闭合 `{`，调用 `subscriber.error(Error::parser("unclosed '{'"))`

#### Scenario: 未闭合的圆括号
- **WHEN** 输入为 `(blue: #0d6efd`（缺少 `)`）
- **THEN** Parser 通过 `check_unclosed_delimiters` 扫描发现未闭合 `(`，调用 `subscriber.error(Error::parser("unclosed '('"))`

#### Scenario: Valid input completes normally
- **WHEN** 输入为 `"a { color: red; }"` 时
- **THEN** Parser 转发所有节点后，`check_unclosed_delimiters` 未发现未闭合分隔符，调用 `subscriber.complete()`
