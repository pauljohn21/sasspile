# Proposal

## Why

当前 reactive-pipeline 实现存在 5 个结构性缺陷，导致无法构建完整的 Sass 编译器：(1) 错误类型为 `Infallible` 无法传播 Lexer/Parser 错误；(2) `SassOp` 返回 `Box<dyn Fn>` 绕过了 rxrust 调度/背压/取消系统；(3) `collect_css` 中残留 `Rc<RefCell>`；(4) Lexer/Parser 完全缺失，`from_string` 是 stub；(5) `AstNode` 是 Evaluator 消费型 AST，无法表达插值/`&`/Maps 等解析树必须承载的语法构造。本次重构对标 Grass 的独立实现路径，一次性解决全部问题。

## What Changes

- **新增 `SassAstNode` 解析树** — 独立于 `AstNode` 的完整 Sass 语法树，支持插值、父选择器 `&`、Maps、Lists、`!default` 标记等
- **新增 lowering 转换层** — `SassAstNode` → `AstNode` 的显式降级 pass
- **新增 Lexer** — 用 `Observable::create` + `scan(LexerState)` 实现流式词法分析，错误类型为泛型 `E`
- **新增 Parser** — 用 `Local::create` + `scan_map(ParserState)` + `flat_map` + `collect` 响应式管道实现流式语法分析，支持 SCSS/SASS/CSS 三种语法，错误通过 `check_unclosed_delimiters` 在流结束后经 `on_error` 通道传播
- **改造 Evaluator** — 每个 `SassOp` 用 `Observable::create` 构建原生算子，替代 `Box<dyn Fn>` 函数指针
- **错误类型从 `Infallible` 改为泛型 `E: crate::Error`** — 错误可经 `on_error` 通道传播
- **消除所有 `Rc<RefCell>`** — 用纯 rxrust 算子实现收集逻辑
- **新增 Filesystem 抽象** — `Fs` trait 处理 `@import` 文件解析（生产用 `StdFs`，测试用 `NullFs`）
- **新增内置函数模块** — `sass:color` (darken/lighten/mix/rgba 等)、`sass:math`、`sass:string`、`sass:list`、`sass:map`
- **新增 `from_string` / `from_path` 公共 API** — 完整替代 stub，产出与 Grass 逐字节一致的 CSS 输出
- **OutputStyle 支持** — Expanded / Compressed / Nested 三种输出格式

## Capabilities

### New Capabilities

- `scss-parse-tree`: 完整 Sass 语法解析树，支持插值、父选择器、Maps、Lists、`!default`、注释保留，作为 Parser 的直接产物
- `parse-lowering`: 显式降级转换层，将 `SassAstNode` 解析树转换为 Evaluator 消费的 `AstNode`，处理插值展开、& 展开、Map/List 转换
- `builtin-modules`: 内置 Sass 模块实现（sass:color、sass:math、sass:string、sass:list、sass:map、sass:meta、sass:selector），覆盖 Bootstrap 所需的所有内置函数

### Modified Capabilities

- `reactive-pipeline`: 核心管线实现彻底改造 — 错误类型从 `Infailable` 变为泛型 `E`，Lexer/Parser 用 `Observable::create` 实现并正确接入管线，所有阶段通过原生 rxrust 算子连接
- `directive-ops`: 每个操作符从 `Box<dyn Fn(AstStream) → AstStream>` 改造为 `Observable::create` 构建的原生算子，接入 rxrust 调度系统
- `multicast-bus`: `CompilerBus` Subject 错误类型从 `Infailable` 变为泛型 `E`，使其能携带错误事件

## Impact

- **代码**: 新增 `src/lexer/`、`src/parser/`、`src/lowering/`、`src/builtin/` 四个目录；改造 `src/eval/`；移除 `src/reactive/ops/` 下的 `Box<dyn Fn>` 分发
- **测试**: 现有 27 个测试需适配新接口（`from_string_ast` 保留为内部 helpers，外部接口改为 `from_string`）。测试套件新增 Lexer/Parser/lowering/builtin 的单元测试
- **依赖**: 继续使用 `rxrust = "1.0.0-rc.5"`，无需新增外部依赖。`codemap` 可选用于错误位置报告
- **API**: `from_string` 和 `from_path` 成为完整功能的公共 API；`from_string_ast` 保留但标记为内部用途
- **文件组织**: 单文件 ≤ 500 行约束下 `lexer/` 和 `parser/` 需拆分为多模块目录
- **向后兼容**: 输出 CSS 与 Grass v0.13.x 逐字节一致（对标 Bootstrap 5.3.8 dist）
