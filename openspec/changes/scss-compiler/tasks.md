# SCSS Compiler Implementation Tasks

每个任务目标：**单一职责、单文件不过大、可独立验证**。

## Phase 1 — Core Pipeline Foundation

- [x] Task 1.1: Project scaffolding — 创建 Cargo.toml (edition 2024, tokio, tracing, thiserror, mietre, serde), 配置 tracing subscriber, 验证 cargo build 通过
- [x] Task 1.2: Value System (`value/`) — mod.rs定义Value枚举(number/string/boolean/null/color/map/arglist/function/calculation/error), number.rs数值与单位, color.rs srgb颜色, ops.rs运算(≤80行), coerce.rs转换(≤60行), ser.rs CSS序列化
- [x] Task 1.3: Source Location (`source/`) — span.rs 定义SourceSource(start, end), pos.rs SourcePosition(line, column), 确保 Clone + Send + Sync + 'static
- [x] Task 1.4: Diagnostics (`diagnostics/`) — diagnostic.rs Diagnostic/DiagnosticBuilder, renderer.rs 带source snippet渲染器, level.rs Level枚举(Error/Warn/Info)

## Phase 2 — Lexer

- [x] Task 2.1: Token定义 (`lexer/token.rs`) — Token枚举(所有TokenType), TokenKind定义, Token携带SourceSpan
- [x] Task 2.2: Lexer实现 (`lexer/lexer.rs`) — 主字符迭代逻辑, 标识符/数字/串/运算符识别, 插值`#{}`处理
- [x] Task 2.3: Indented syntax 支持 (`lexer/sass_syntax.rs`) — .sass 缩进语法(Indent/Dedent)
- [x] Task 2.4: Lexer 测试 — 基于spec场景的#[test]用例

## Phase 3 — Parser

- [ ] Task 3.1: AST定义 (`parser/ast.rs`) — Node/Rule/Declaration/AtRule等枚举, 指令变体(Use/Import/Forward/Mixin/Include/Function/Return/If/Else/For/Each/While/Extend/AtRoot/Media/Supports/Content/Debug/Warn/Error)
- [ ] Task 3.2: Parser实现 (`parser/parser.rs`) — 递归下降解析器, 嵌套规则展开
- [ ] Task 3.3: 插值解析 (`parser/interpolation.rs`) — 在selector/property/value/string中的`#{}`
- [ ] Task 3.4: 错误恢复 (`parser/recovery.rs`) — 非致命错误时尝试同步并继续

## Phase 4 — Semantic Analysis

- [ ] Task 4.1: 符号表 (`semantic/symbol_table.rs`) — 作用域栈(Global/Local/Param), 名称查找与遮蔽规则
- [ ] Task 4.2: 模块解析 (`semantic/module.rs`) — @use/@forward依赖图, 循环依赖检测
- [ ] Task 4.3: 扩展验证 (`semantic/extend.rs`) — @extend目标存在性检查
- [ ] Task 4.4: 定义收集 (`semantic/definitions.rs`) — Mixin与Function注册表，重复检测

## Phase 5 — Expression Evaluation

- [ ] Task 5.1: 求值器框架 (`eval/evaluator.rs`) — EvalContext与环境
- [ ] Task 5.2: 运算符实现 (`eval/ops.rs`) — 算术/字符串/比较/逻辑, 单位一致性检查
- [ ] Task 5.3: 函数调用 (`eval/functions.rs`) — 用户函数调用栈, 内置函数分发
- [ ] Task 5.4: 列表/Map访问 (`eval/collections.rs`) — nth/key/dot访问

## Phase 6 — Built-in Modules

- [ ] Task 6.1: `sass:color` (`builtin/sass_color.rs`) — 颜色操作函数(lighten/darken/saturate/...)
- [ ] Task 6.2: `sass:math` (`builtin/sass_math.rs`) — 数学函数(div/round/sin/cos/...)
- [ ] Task 6.3: `sass:list` (`builtin/sass_list.rs`) — 列表操作(join/append/length/nth/...)
- [ ] Task 6.4: `sass:map` (`builtin/sass_map.rs`) — Map操作(get/merge/deep-merge/...)
- [ ] Task 6.5: `sass:string` (`builtin/sass_string.rs`) — 字符串操作(split/slice/unquote/...)
- [ ] Task 6.6: `sass:meta` (`builtin/sass_meta.rs`) — 元信息函数(type-of/call/get-function/...)
- [ ] Task 6.7: 模块注册 (`builtin/mod.rs`) — `HashMap<&str, Box<dyn SassFn>>`注册

## Phase 7 — CSS Generation

- [ ] Task 7.1: Generator框架 (`css/generator.rs`) — OutputStyle配置
- [ ] Task 7.2: 规则展开 (`css/rules.rs`) — 嵌套选择器展开为平面规则
- [ ] Task 7.3: At-rule输出 (`css/atrules.rs`) — @media, @supports, @import
- [ ] Task 7.4: Source Maps (`css/sourcemap.rs`) — v3 JSON格式输出

## Phase 8 — Incremental Compilation

- [ ] Task 8.1: Watch-based响应式环境 (`incremental/env.rs`) — watch::Sender<Value>/watch::Receiver
- [ ] Task 8.2: 依赖图 (`incremental/depgraph.rs`) — 变量→依赖节点的映射
- [ ] Task 8.3: 缓存层 (`incremental/cache.rs`) — 基于SourceSpan的Arc<Value>缓存
- [ ] Task 8.4: 变更传播 (`incremental/propagate.rs`) — 上游变量变更触发下游重编译

## Phase 9 — Pipeline Orchestration

- [ ] Task 9.1: 7-stage Tokio pipeline (`pipeline/mod.rs`) — tokio::spawn任务 + mpsc连接
- [ ] Task 9.2: 背压与取消 (`pipeline/backpressure.rs`) — bounded channel, CancellationToken
- [ ] Task 9.3: 并发编译 (`pipeline/concurrent.rs`) — 多入口文件并行，共享模块缓存
- [ ] Task 9.4: 进度跟踪 (`pipeline/tracing.rs`) — 每个stage tracing::info!/tracing::debug!

## Phase 10 — CSS4 Colors（可选后续阶段）

- [ ] Task 10.1: 解析 color()/lab()/oklab()/oklch()
- [ ] Task 10.2: color-mix() 实现
- [ ] Task 10.3: 相对颜色语法 from <color>
- [ ] Task 10.4: light-dark() / hwb()

## Phase 11 — Testing & Sass Spec Integration

- [ ] Task 11.1: HRX reader integration (`tests/hrx_loader.rs`) — 读取sass-spec HRX文件树
- [ ] Task 11.2: Spec运行器 (`tests/spec_runner.rs`) — 遍历spec目录，执行每个用例并比对expected.css
- [ ] Task 11.3: CSS4 color skip清单 — 使用既有css4_color_skip.rs跳过462文件
- [ ] Task 11.4: 全量回归测试 — 目标：non-color用例100%通过

## Phase 12 — Documentation & Release

- [ ] Task 12.1: 编写README（项目背景、用法、架构图）
- [ ] Task 12.2: 编写examples/（基础用法、并发编译、增量编译）
- [ ] Task 12.3: Cargo doc文档
- [ ] Task 12.4: 发布至crates.io（版本0.1.0）

## 完成标准
- cargo build 通过
- cargo test 全部通过
- cargo clippy 无warning
- sass-spec non-color用例100%通过
- 单文件≤400行(源码+测试分别计算)
- 零eprintln!/println!(仅tracing)
