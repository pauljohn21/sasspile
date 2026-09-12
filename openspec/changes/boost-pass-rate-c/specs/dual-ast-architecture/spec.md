## ADDED Requirements

### Requirement: SCSS 文件产出 ScssAst
Sass 编译器 SHALL 解析 `.scss` / `.sass` 文件为 `ScssAst`（包含 `ScssNode` 枚举）。

#### Scenario: .scss 文件解析
- **WHEN** 编译 `input.scss` 文件
- **THEN** 解析器 SHALL 产出 `Parsed::Scss(ScssAst { nodes: Vec<ScssNode> })`

#### Scenario: .sass 文件解析
- **WHEN** 编译 `input.sass` 文件（缩进语法）
- **THEN** 解析器 SHALL 产出 `Parsed::Scss(ScssAst { nodes: Vec<ScssNode> })`

#### Scenario: SCSS 特有语法
- **WHEN** 解析包含 `$var`、`@if`、`@mixin`、`@include` 的 `.scss` 文件
- **THEN** ScssAst SHALL 包含对应的 `ScssNode::Variable`、`ScssNode::If`、`ScssNode::MixinDef`、`ScssNode::Include` 节点

---

### Requirement: CSS 文件产出 CssAst
Sass 编译器 SHALL 解析 `.css` 文件为 `CssAst`（包含 `CssNode` 枚举）。

#### Scenario: .css 文件解析
- **WHEN** 编译 `input.css` 文件
- **THEN** 解析器 SHALL 产出 `Parsed::Css(CssAst { nodes: Vec<CssNode> })`

#### Scenario: CSS Nesting 解析
- **WHEN** 解析包含嵌套规则 `a { .b { color: blue } }` 的 `.css` 文件
- **THEN** CssAst SHALL 包含 `CssNode::Rule { body: [CssNode::Rule { ... }] }` 嵌套结构

#### Scenario: Sass 语法在 CSS 文件中拒绝
- **WHEN** 解析包含 `$var` 或 `@if` 的 `.css` 文件
- **THEN** 解析器 SHALL 返回 Parse 错误（Sass 语法不允许在 CSS 文件中）

---

### Requirement: ScssNode 与 CssNode 类型互斥
ScssNode SHALL 包含 Sass 特有变体（Variable、If、For、Each、While、MixinDef、FunctionDef、Include、Return、Content、Use、Forward、Extend、AtRoot、Warn、Debug、Error）；CssNode SHALL 仅包含 CSS 原生变体（Rule、Decl、Comment、AtRule、Import）。

#### Scenario: CssNode 无 Variable 变体
- **WHEN** 编译引用 `CssNode::Variable` 的代码
- **THEN** Rust 编译器 SHALL 报错（variant not found）

#### Scenario: ScssNode 有 Variable 变体
- **WHEN** 编译引用 `ScssNode::Variable` 的代码
- **THEN** Rust 编译器 SHALL 通过

---

### Requirement: ScssEvaluator 展平语义
ScssEvaluator SHALL 展开嵌套规则（选择器组合）并产出扁平 CssNode 树。

#### Scenario: 嵌套规则展平
- **WHEN** ScssEvaluator 处理 `a { .b { color: blue } }`
- **THEN** 输出 SHALL 包含 `CssNode::Rule { selector: "a .b", declarations: [...], children: [] }`

#### Scenario: @media 提升
- **WHEN** ScssEvaluator 处理 `a { @media (min-width: 768px) { color: blue } }`
- **THEN** 输出 SHALL 将 @media 提升到外层，内部规则选择器为 `a`

---

### Requirement: CssEvaluator 嵌套保留语义
CssEvaluator SHALL 保留嵌套结构（不组合选择器）并产出嵌套 CssNode 树。

#### Scenario: 嵌套规则保留
- **WHEN** CssEvaluator 处理 `a { .b { color: blue } }`
- **THEN** 输出 SHALL 包含 `CssNode::Rule { selector: "a", declarations: [], children: [CssNode::Rule { selector: ".b", ... }] }`

#### Scenario: @media 不提升
- **WHEN** CssEvaluator 处理 `a { @media (min-width: 768px) { color: blue } }`
- **THEN** 输出 SHALL 将 @media 保留在 `a {}` 的 children 中

#### Scenario: @import 透传
- **WHEN** CssEvaluator 处理 `@import "other.css"`
- **THEN** 输出 SHALL 包含 `CssNode::AtImport { url: "other.css" }`

---

### Requirement: 统一 Serializer 无 mode 参数
Serializer SHALL 按 CssNode 树的结构原样输出，无需知道来源是 SCSS 还是 CSS。

#### Scenario: SCSS 产物序列化
- **WHEN** 序列化来自 ScssEvaluator 的扁平 CssNode 树
- **THEN** 输出 SHALL 为展平 CSS（`a .b { color: blue }`）

#### Scenario: CSS 产物序列化
- **WHEN** 序列化来自 CssEvaluator 的嵌套 CssNode 树
- **THEN** 输出 SHALL 为嵌套 CSS（`a { .b { color: blue } }`）

---

### Requirement: Reactor 管线分派
Reactor SHALL 根据 Parsed 类型选择对应 Evaluator，Serializer 调用无需传递 mode。

#### Scenario: SCSS 文件管线
- **WHEN** Reactor 处理 `.scss` 文件
- **THEN** SHALL 调用 `ScssEvaluator::evaluate` → `Serializer::serialize`

#### Scenario: CSS 文件管线
- **WHEN** Reactor 处理 `.css` 文件
- **THEN** SHALL 调用 `CssEvaluator::evaluate` → `Serializer::serialize`

---

### Requirement: 独立演化能力
新增 CSS 特性 SHALL 仅修改 CssNode + CssEvaluator，不影响 SCSS 路径的任何测试。

#### Scenario: CSS 新特性不影响 SCSS
- **WHEN** 为 CSS5 新增 `CssNode::NewFeature` 变体
- **THEN** SCSS 测试套件 SHALL 全部通过（无编译错误、无行为变化）

#### Scenario: SCSS 新特性不影响 CSS
- **WHEN** 为 SCSS4 新增 `ScssNode::NewFeature` 变体
- **THEN** CSS 测试套件 SHALL 全部通过（无编译错误、无行为变化）

