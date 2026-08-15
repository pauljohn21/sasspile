# SCSS Compiler Implementation Tasks

每个任务目标：**单一职责、单文件不过大、可独立验证**。

## Phase 1 — Core Pipeline Foundation

### Task 1.1: Project scaffolding
- 创建 `Cargo.toml` (edition 2024, `tokio`, `tracing`, `thiserror`, `miette`, `serde`)
- 配置 `tracing` subscriber
- 验证 `cargo build` 通过

### Task 1.2: Value System (`value_system/`)
- `mod.rs`: 定义 `Value` 枚举（Number, String, Boolean, Null, Color, List, Map, ArgList, Function, Calculation, Error）
- `number.rs`: 数值与单位表示
- `color.rs`: srgb 颜色表示
- `ops.rs`: 算术/比较/逻辑运算（<= 80 行）
- `coerce.rs`: 类型转换（<= 60 行）
- `ser.rs`: CSS 序列化
- 单元测试：等值性、序列化输出

### Task 1.3: Source Location (`source/`)
- `span.rs`: `SourceSpan` 定义（start, end）
- `pos.rs`: `SourcePosition` (line, column)
- 确保 `Clone + Send + Sync + 'static`

### Task 1.4: Diagnostics (`diagnostics/`)
- `diagnostic.rs`: `Diagnostic`, `DiagnosticBuilder`
- `renderer.rs`: 带 source snippet 的渲染器
- `level.rs`: `Level` 枚举 (Error/Warn/Info)

---

## Phase 2 — Lexer

### Task 2.1: Token 定义 (`lexer/token.rs`)
- `Token` 枚举 (所有 TokenType)
- `TokenKind` 定义
- `Token` 携带 `SourceSpan`

### Task 2.2: Lexer 实现 (`lexer/lexer.rs`)
- 主字符迭代逻辑
- 标识符、数字、串、运算符识别
- 插值 `#{}` 处理

### Task 2.3: Indented syntax 支持 (`lexer/sass_syntax.rs`)
- `.sass` 缩进语法（Indent/Dedent）

### Task 2.4: Lexer 测试
- 基于 spec 中场景的 `#[test]` 用例

---

## Phase 3 — Parser

### Task 3.1: AST 定义 (`parser/ast.rs`)
- `Node`, `Rule`, `Declaration`, `AtRule` 等枚举
- 指令变体：Use, Import, Forward, Mixin, Include, Function, Return, If, Else, For, Each, While, Extend, AtRoot, Media, Supports, Content, Debug, Warn, Error

### Task 3.2: Parser 实现 (`parser/parser.rs`)
- 递归下降解析器
- 嵌套规则展开

### Task 3.3: 插值解析 (`parser/interpolation.rs`)
- 在 selector/property/value/string 中的 `#{}`

### Task 3.4: 错误恢复 (`parser/recovery.rs`)
- 非致命错误时尝试同步并继续

---

## Phase 4 — Semantic Analysis

### Task 4.1: 符号表 (`semantic/symbol_table.rs`)
- 作用域栈（Global, Local, Param）
- 名称查找与遮蔽规则

### Task 4.2: 模块解析 (`semantic/module.rs`)
- `@use` / `@forward` 依赖图
- 循环依赖检测

### Task 4.3: 扩展验证 (`semantic/extend.rs`)
- `@extend` 目标存在性检查

### Task 4.4: 定义收集 (`semantic/definitions.rs`)
- Mixin 与 Function 注册表，重复检测

---

## Phase 5 — Expression Evaluation

### Task 5.1: 求值器框架 (`eval/evaluator.rs`)
- `EvalContext` 与环境

### Task 5.2: 运算符实现 (`eval/ops.rs`)
- 算术、字符串、比较、逻辑
- 单位一致性检查

### Task 5.3: 函数调用 (`eval/functions.rs`)
- 用户函数调用栈
- 内置函数分发

### Task 5.4: 列表/Map 访问 (`eval/collections.rs`)
- nth, key, dot 访问

---

## Phase 6 — Built-in Modules

### Task 6.1: `sass:color` (`builtin/color.rs`)
- 颜色操作函数（lighten/darken/saturate/...）

### Task 6.2: `sass:math` (`builtin/math.rs`)
- 数学函数（div/round/sin/cos/...）

### Task 6.3: `sass:list` (`builtin/list.rs`)
- 列表操作（join/append/length/nth/...）

### Task 6.4: `sass:map` (`builtin/map.rs`)
- Map 操作（get/merge/deep-merge/...）

### Task 6.5: `sass:string` (`builtin/string.rs`)
- 字符串操作（split/slice/unquote/...）

### Task 6.6: `sass:meta` (`builtin/meta.rs`)
- 元信息函数（type-of/call/get-function/...）

### Task 6.7: 模块注册 (`builtin/registry.rs`)
- `HashMap<&str, Box<dyn SassFn>>` 注册

---

## Phase 7 — CSS Generation

### Task 7.1: Generator 框架 (`css/generator.rs`)
- `OutputStyle` 配置

### Task 7.2: 规则展开 (`css/rules.rs`)
- 嵌套选择器展开为平面规则

### Task 7.3: At-rule 输出 (`css/atrules.rs`)
- `@media`, `@supports`, `@import`

### Task 7.4: Source Maps (`css/sourcemap.rs`)
- v3 JSON 格式输出

---

## Phase 8 — Incremental Compilation

### Task 8.1: Watch-based 响应式环境 (`incremental/env.rs`)
- `watch::Sender<Value>` / `watch::Receiver`

### Task 8.2: 依赖图 (`incremental/depgraph.rs`)
- 变量 → 依赖节点的映射

### Task 8.3: 缓存层 (`incremental/cache.rs`)
- 基于 SourceSpan 的 `Arc<Value>` 缓存

### Task 8.4: 变更传播 (`incremental/propagate.rs`)
- 上游变量变更触发下游重编译

---

## Phase 9 — Pipeline Orchestration

### Task 9.1: 7-stage Tokio pipeline (`pipeline/mod.rs`)
- `tokio::spawn` 任务 + `mpsc` 连接

### Task 9.2: 背压与取消 (`pipeline/backpressure.rs`)
- bounded channel, `CancellationToken`

### Task 9.3: 并发编译 (`pipeline/concurrent.rs`)
- 多入口文件并行，共享模块缓存

### Task 9.4: 进度跟踪 (`pipeline/tracing.rs`)
- 每个 stage `tracing::info!` / `tracing::debug!`

---

## Phase 10 — CSS4 Colors（可选后续阶段）

### Task 10.1: 解析 `color()` / `lab()` / `oklab()` / `oklch()`
### Task 10.2: `color-mix()` 实现
### Task 10.3: 相对颜色语法 `from <color>`
### Task 10.4: `light-dark()` / `hwb()`

---

## Phase 11 — Testing & Sass Spec Integration

### Task 11.1: HRX reader integration (`tests/hrx_loader.rs`)
- 读取 sass-spec HRX 文件树

### Task 11.2: Spec 运行器 (`tests/spec_runner.rs`)
- 遍历 spec 目录，执行每个用例并比对 expected.css

### Task 11.3: CSS4 color skip 清单
- 使用既有 `css4_color_skip.rs` 跳过 462 文件

### Task 11.4: 全量回归测试
- 目标：non-color 用例 100% 通过

---

## Phase 12 — Documentation & Release

### Task 12.1: 编写 README（项目背景、用法、架构图）
### Task 12.2: 编写 `examples/` （基础用法、并发编译、增量编译）
### Task 12.3: Cargo doc 文档
### Task 12.4: 发布至 crates.io（版本 0.1.0）

---

## 完成标准
- `cargo build` 通过
- `cargo test` 全部通过
- `cargo clippy` 无 warning
- sass-spec non-color 用例 100% 通过
- 单文件 ≤ 400 行（源码 + 测试分别计算）
- 零 `eprintln!` / `println!`（仅 tracing）
