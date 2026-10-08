# Serialize Specification

## Purpose

将 `CssStmt` 流格式化为 CSS 字符串输出序列，支持 Expanded/Compressed/Nested 三种 OutputStyle。由 `CompileBuilder::build` 内部构建。

## Requirements

### Requirement: OutputStyle 支持
`CompileBuilder::build` 的序列化阶段 SHALL 支持三种 style：`Expanded`（缩进换行）、`Compressed`（最小空白）、`Nested`（子规则视觉缩进）。`indent_width` 控制缩进空格数（默认 2）。`suppress_charset` 控制是否输出 `@charset "UTF-8";` 头。通过 Builder `.serialize_style()` 方法配置。

#### Scenario: Expanded style output
- **WHEN** 输入 `Rule(".a", [Decl("color", "red")])` style=Expanded
- **THEN** SHALL 输出 `".a {\n  color: red;\n}\n"`

#### Scenario: Compressed style output
- **WHEN** 相同输入但 style=Compressed
- **THEN** SHALL 输出 `".a{color:red}"`

#### Scenario: Charset suppression
- **WHEN** suppress_charset = true
- **THEN** SHALL 不包含 `@charset "UTF-8";`

#### Scenario: Charset inclusion by default
- **WHEN** suppress_charset = false 且有非空输出
- **THEN** SHALL 在输出开头包含 `@charset \"UTF-8\";\n`

### Requirement: 增量缩进管理
Expanded/Nested 模式下 Serializer SHALL 维护当前 `depth: usize`。每进入 Rule/Media/Supports 块 depth 递增，退出时递减。缩进空格 = `indent_width * depth`。Decl 缩进比所在块深一层。

#### Scenario: Nested rules with correct indentation
- **WHEN** 输入 `.parent { .child { color: red; } }` style=Expanded indent_width=2
- **THEN** SHALL 产生 ".parent {\n  .child {\n    color: red;\n  }\n}\n"

### Requirement: 块展开规则
Rule/Media/Supports 序列化 SHALL 遵循 `{selector/query} {\n {inner_at_depth+1}\n {depth_indent}}` 模式。Decl 末尾 SHALL 跟随 `;\n`。

#### Scenario: Media block formatting
- **WHEN** 输入 `Media("screen", [Decl("width", "768px")])`
- **THEN** SHALL 输出 "@media screen {\n  width: 768px;\n}\n"

### Requirement: 空规则过滤
Serializer SHALL 过滤 inner 全为空的 Rule/Media/Supports。不出现在输出中。

#### Scenario: Empty rule filtered
- **WHEN** 输入 `Rule(".empty", [])`
- **THEN** SHALL 不出现在输出中

### Requirement: Observable 流式输出
序列化阶段输出的流 SHALL 每个 CssStmt 对应一个 String 元素。流结束时自动完成。

#### Scenario: Streaming output
- **WHEN** CssStmt 流发射 Rule 后跟 Media
- **THEN** String 流 SHALL 分别发射对应格式字符串
