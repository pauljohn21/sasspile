# Parse Lowering Specification

## Purpose

定义从 `SassAstNode` 解析树到 `AstNode` 消费型 AST 的 lowering 转换层，处理插值展开、父选择器展开、Map/List 转换、`!default` 语义降级等变换，使 Evaluator 无需感知源码级语法细节。

## Requirements

### Requirement: 系统 SHALL 提供 `lower_to_ast` 函数
系统 SHALL 提供 `pub fn lower_to_ast(node: SassAstNode, ctx: &LoweringContext) -> Result<AstNode>` 函数，将单个 SassAstNode 递归转换为 AstNode。

#### Scenario: 简单样式声明降级
- **WHEN** 输入 `SassAstNode::StyleDecl { prop: "color", value: "red" }`
- **THEN** 产出 `AstNode::StyleDecl { property: "color", value: "red" }`

#### Scenario: Sass Map 转换为 Value
- **WHEN** 输入 `SassAstNode::MapLiteral(vec![("blue", "#0d6efd"), ("red", "#dc3545")])`
- **THEN** 产出 `Value::Map`

#### Scenario: Sass List 转换为 Value
- **WHEN** 输入 `SassAstNode::ListLiteral(vec![...])`
- **THEN** 产出 `Value::List`

### Requirement: 插值表达式 SHALL 在 lowering 时展开
`SassAstNode::Interpolated(expr)` 在 lowering 时 SHALL 根据当前变量环境解析为具体字符串值。无法解析的插值 SHALL 返回错误。

#### Scenario: 变量插值展开
- **WHEN** 输入 `SassAstNode::Interpolated("$prefix")` 且环境中 `$prefix = "bs-"`
- **THEN** 产出字符串 `"bs-"`

#### Scenario: 嵌套属性插值
- **WHEN** 输入 `SassAstNode::Interpolated("#{$prefix}btn-color")` 且 `$prefix = "bs-"`
- **THEN** 产出字符串 `"bs-btn-color"`

### Requirement: 父选择器 SHALL 在 lowering 时展开为完整选择器
`SassAstNode::ParentSelector` 在 lowering 时 SHALL 结合祖先选择器栈拼接出完整后代选择器。例如祖先为 `"a"` 时，`&:hover` 展开为 `"a:hover"`。

#### Scenario: 单层父选择器
- **WHEN** 规则 `.btn { &:hover { color: red; } }` 展开
- **THEN** 父选择器 `&:hover` 展开为 `.btn:hover`

#### Scenario: 多层嵌套父选择器
- **WHEN** 规则 `.card { .body { &:focus { ... } } }` 在 lowering
- **THEN** `&:focus` 展开为 `.card .body:focus`

### Requirement: `!default` 语义 SHALL 在 lowering 处理
当变量声明带有 `has_default: true` 且环境中已存在同名变量时，lowering SHALL 丢弃该声明（保持已有绑定值不变）。

#### Scenario: 默认值被覆盖
- **WHEN** 环境中已有 `$color: blue`，遇到 `$color: red !default`
- **THEN** 该声明被静默丢弃，不产生 Bind 事件

#### Scenario: 默认值生效
- **WHEN** 环境中不存在 `$color`，遇到 `$color: red !default`
- **THEN** 正常产出 Bind 事件

### Requirement: lowering 阶段 SHALL 正确传播错误
lowering 过程中遇到的任何错误（未定义变量、类型不匹配等） SHALL 返回 `Err(crate::Error)`，而非 panic。

#### Scenario: 未定义变量插值
- **WHEN** lowering `SassAstNode::Interpolated("$undefined")` 且变量不存在
- **THEN** 返回 `Err(crate::Error)`，提示未定义变量
