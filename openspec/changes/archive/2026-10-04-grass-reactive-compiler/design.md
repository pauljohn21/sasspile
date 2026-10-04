# Design / 设计

## Context / 背景

See proposal.md - Why for motivation. / 动机详见 proposal.md 的 Why 章节。

**当前 grass 架构的两个核心问题 / Two core problems in current architecture:**
1. `Rc<RefCell<Environment>>` 贯穿所有求值器，导致运行时 borrow-check panic 和内部可变性——违反 Rust 所有权原则
2. 2500 行的 `visit_stmt` 巨石函数——不可测试、不可拆分、添加新指令必须修改该函数
3. 无法流式输出 —— 必须完整解析+求值后才能产出任何 CSS

**关键约束 / Key constraints:**
- `grass` 使用 `codemap` (`SourceMap`) 做错误报告——源码位置必须穿过所有管道阶段
- `Value` 枚举 (`grass::Value`) 已有 Number、String、List、Map、Color、Boolean、Null、Lambda、Calculated 变体
- `ModuleSystem` 已按 `Module` 对象组织（包含 member map）——响应式设计将其包装为 `Rc<Module>` 通过多播共享

## Goals / Non-Goals / 目标与非目标

**Goals / 目标:**
- 消除代码库中所有 `RefCell`/`Cell` ——通过 `cargo clippy` 验证，`src/` 中找不到 `std::cell` 使用
- 支持流式输出：首字节在完整编译之前发出
- 每条 `@`-指令都是独立算子——可组合、可测试、可替换
- 100% 向后兼容 API：`from_string()`、`from_path()` 产生逐字节一致输出
- 模块系统（`@use`/`@forward`/`@import` 遗留）通过多播事件工作

**Non-Goals / 非目标:**
- 移除或替换 `codemap` 依赖
- 改变 `Value` 枚举语义或添加新值类型
- 移除 `ModuleSystem` 抽象或将其与 `CompilerBus` 合并
- 实现增量编译或热重载
- 改变选择器 AST 或 MIME 类型处理

## Decisions / 关键决策

### Decision 1: CompilerBus 作为唯一协调原语 / CompilerBus as single coordination primitive

**方案 / Approach:** 一个 `CompilerBus` 结构体包含三个 `Subject` 实例（通过 `Subject::new()` 创建）作为多播通道：
- `var_events: Subject<ValueEvent>` —— 变量绑定下游传播
- `module_events: Subject<ModuleEvent>` —— 模块编译完成通知
- `css_scope_subject: Subject<ScopeEvent>` —— 进入/退出作用域以供 CSS 收集

每次编译创建一个 `CompilerBus`，并通过 clone（`Rc<InnerBus>`）与所有管道阶段共享。

**被否定的方案 / Alternatives rejected:**
- **A**: 保留 `RefCell<Environment>` + 手动 Observer 模式——仍存在内部可变性。❌
- **B**: 标准 tokio mpsc 通道——mpsc 不是多播（每个事件只能被一个消费者接收），需要 fanout 逻辑。❌
- **C**: 自定义 `Vec<Callback>` 回调——需要手动管理背压，比 rxrust 更多模板代码。❌

**选择理由 / Rationale:** `Subject::new()` 天然支持 Publish 语义（经典多播源——每个订阅观察者获得所有条目）。订阅模型替代了传统的"拉取"模型：不再轮询环境，而是自动接收推送。

### Decision 2: SassOp trait + 所有权驱动的操作符生命周期 / SassOp trait with ownership-based operator lifecycle

**方案 / Approach:** 定义 `pub trait SassOp`，包含方法 `fn into_operator(self, ctx: Rc<EvalContext>) -> Box<dyn Fn(Observable<AstNode>) -> Observable<AstNode>>>`。每个语句类型的 AST 节点实现此 trait。`self` 按值取走（move），强制单所有者语义；`ctx` 为 `Rc<EvalContext>`，嵌套求值时通过 clone 共享。

操作符签名是 `AstNode → AstNode`（中间层），而非直接 `AstNode → CssStmt`，原因：
1. `@`-指令可能需要多次 AST 展开（如 `@for` 展开为 N 个 body 副本）
2. 需要在 CSS 输出前处理变量分析、mixin 展开等前置遍历

**被否定的方案 / Alternatives rejected:**
- **A**: 返回 `Vec<CssStmt>` —— 破坏流式，必须全量收集到内存。❌
- **B**: 裸 `Box<dyn Fn>` 无 trait —— 无法在模块注册表中传递。❌
- **C**: 中央注册表 enum dispatch —— 添加指令必须修改注册表。❌

### Decision 3: flat_map_extract_items 和 switch_map_extract_items 作为扩展操作符

**系统 SHALL** 为 `Observable<T>` 提供两个新方法：
- `flat_map_extract_items` —— `flat_map(iter_into_local)` 的类型别名，将 `Observable<Vec<T>>` 转换为 `Observable<T>`
- `switch_map_extract_items` —— `switch_map` + `flat_map_extract_items` 组合，确保"仅最新"语义 + 展开

**原因:** Sass 列表/映射操作会产生 `Vec<Value>`，需要一层展平为独立 CSS 声明。集中定义这两个组合子可避免在 20 个操作符实现中重复手写 `flat_map` 逻辑。

### Decision 4: 作用域预分析 pass / Scope pre-analysis pass

**方案:** 在 AstNode 流求值前，运行一个预分析 pass 为每个 AstNode 分配 `scope_id`（单调递增 u64，深度层级格式：`parent_idx * 1000 + local_counter`）。多播过滤器使用它进行静态作用域查找：`var_events.filter(move |e| e.scope_id == current_scope || e.scope_id < current_scope)`。

**原因:** 静态作用域信息替代了共享的可变环境栈。multicast filter 可以利用作用域 ID 高效查找绑定，无需任何共享可变性。

### Decision 5: 媒体查询包装通过作用域驱动 buffer / Media scope buffering

**方案:** AstNode 求值遇到 `@media` 时，向 `css_scope_subject` 发射 `ScopeEvent::Enter`。内部语句被收集到缓冲区中。`ScopeEvent::Exit` 触发将所有已收集的 CssStmts 包装成单条 `CssStmt::Media` 并发射。

**原因:** 保持了流式——outer ScopeEvents 携带边界信息，内部操作符无需对整个 AST 进行两遍处理。

## Risks / Trade-offs / 风险与权衡

- **[风险] rxrust `Local::from_iter` 对每个订阅克隆迭代器** → 对于流式，Token 小，可接受。如果分析显示分配压力，切换到 `Subject` 推送。
- **[风险] 编译时间可能增加 2-5 倍** → 架构改进的合理权衡。`Box<dyn Observable>` 可能有虚表开销，但实际测量前假设影响可忽略。
- **[风险] `from_string()` 向后兼容** → spectral 测试套件覆盖此要求。grass v0.13.8 的所有测试用例须通过逐字节对比。
- **[风险] `switch_map` 闭包内的 panic 定位** → tracing span 提供上下文。使用 `RUST_LOG=grass=trace,compiler_bus=trace`。
- **[风险] Subscription 泄漏** → RxRust 通过 `CompositeSubscription: Drop` 自动清理。所有订阅在 drop 时关闭。
- **[风险] 深度嵌套导致的栈溢出** → 当前代码有相同风险。Rust 栈帧仍会增长。未来：在 EvalContext 中添加栈深守卫。

## Migration Plan / 迁移计划

**高层策略:** `ReactiveCompiler` 逐步替换 `grass/crates/compiler/src/` 中的现有实现。

1. 在 `compiler/src/reactive/` 下创建完整实现
2. `grass::Compiler::from_string` 改为调用 `ReactiveCompiler`——旧保留到所有测试通过
3. 用 `CompilerBus.module_events` 订阅替换 `Rc<RefCell<ModuleSystem>>`
4. `cargo test -p grass` + spectral 测试套件全部通过

回退策略：若任何 spec 测试失败，旧 `grass::from_string` 在 feature flag 后仍可启用。

## Open Questions / 待定问题

无 —— spec + design + tasks 已形成完整规格说明。实现按任务分解进行。

## Operator Mapping Reference / 操作符映射参考表

实现 MUST 为每个 SassOp 使用对应的 rxrust 操作符：

| @-directive / 指令 | Rx operator / 操作符 | Rationale / 理由 |
|---|---|---|
| `@if`/`@else` | `switch_map` | 选择一个分支，取消其他分支 |
| `@for`/`through` | `flat_map(iter_into_local)` | 迭代范围，发射 body N 次 |
| `@each` | `flat_map(iter_into_local)` | 迭代列表/映射项 |
| `@while` | `unfold()` with termination | 可变循环状态 |
| `@mixin`/`@function` | `tap()` | 注册定义，流原样通过 |
| `@include`/`@function call` | `switch_map` | 展开 body，若重新进入则取消前一次 |
| `@use`/`@forward` | `and_then()` (async boundary) | 编译模块，合并环境 |
| `@media`/`@supports` | `buffer(openings).map(wrap_scope)` | 收集，包装为单 stmt |
| `@error` | `on_error` channel | 终止管道 |
| `@warn`/`@debug` | `tap()` | 诊断输出，流不变 |
| `@content` | `switch_map` (content injection) | 替换占位符 |
| `@at-root` | N/A（pass-through） | 已在 AstNode 流中压平 |
| `@extend` | Pre-analysis pass + map | 在 CSS 发射前解析选择器 |

## Critical Path: String input → CSS output / 关键路径追踪

逐步追踪管道以确保所有 observer 正确连接：

1. `ReactiveCompiler::compile(input, opts)` 创建：
   - `bus = Rc::new(CompilerBus::new())`
   - `chars = Observable::from_iter(input.chars())`

2. `chars.map(|c| normalize(c))` → `Observable<char>`（统一 `\r\n`、`\x0C` 为 `\n`）

3. `.scan(LexerState::new(), |state, c| state.feed(c))` → `Observable<Option<Token>>`
   - `.filter_map(Option::Some)` → `Observable<Token>`

4. `.scan(ParserState::new(), |state, tok| state.feed(tok))` → `Observable<Option<AstNode>>`
   - `.filter_map(Option::Some)` → `Observable<AstNode>`

5. `.scan(EvaluatorState::new(ctx), |state, node| state.visit(node))` → `Observable<Option<AstNode>>`
   - 在 `visit` 中调用 `SassOp::into_operator`，返回闭包变换流
   - 闭包内部调用 `ctx.bus.ver("$name")` 查找变量
   - 变量定义发射 `ValueEvent::Bind` 到 `var_events`
   - `.filter_map(Option::Some)` → `Observable<CssStmt>`

6. `.scan(SerializerState::new(style), |state, stmt| state.serialize(stmt))` → `Observable<Option<String>>`
   - `.filter_map(Option::Some)` → `Observable<String>`（CSS chunks）

7. 订阅者收到 chunks —— 编译 backpressure 通过 `CompositeSubscription` drop 清理。

## Key Type Signatures / 关键类型签名

```rust
// CompilerBus —— 共享多播协调器
pub struct CompilerBus {
    var_events: Subject<ValueEvent>,
    module_events: Subject<ModuleEvent>,
    css_scope_subject: Subject<ScopeEvent>,
}

// ValueEvent —— 变量绑定时发射
pub enum ValueEvent {
    Bind { name: String, value: Value, scope_id: u64 },
}

// ModuleEvent —— 模块编译完成时发射
pub enum ModuleEvent {
    Constructed { name: String, module: Rc<Module> },
}

// ScopeEvent —— 作用域边界
pub enum ScopeEvent {
    Enter { scope_id: u64, kind: ScopeKind },
    Exit { scope_id: u64 },
}

// EvalContext —— 替换 Environment，绑定 bus + scope
pub struct EvalContext {
    bus: Rc<CompilerBus>,
    scope_id: u64,
}

// SassOp trait —— 每个语句类型为独立算子
pub trait SassOp {
    fn into_operator(self, ctx: Rc<EvalContext>) -> Box<dyn Fn(Observable<AstNode>) -> Observable<AstNode>>;
}

// Pipeline compiler —— 连接一切
pub struct ReactiveCompiler {
    bus: Rc<CompilerBus>,
}
```

## Module Mapping: Current → Reactive / 模块映射表：现有 → 响应式

| 当前文件 | 响应式用途 |
|---|---|
| `src/lib.rs` 入口 | 保持：`pub use compiler::{from_string, from_path, Options}` |
| `src/evaluate/visitor.rs` (2500 行) | **拆分为** 每个 `SassOp` 实现一个文件 + dispatch runner |
| `src/evaluate/eval.rs` (Value 操作) | 基本不变——纯函数无需 Observable |
| `src/evaluate/env.rs` (Environment 结构体) | **替换为** `EvalContext { bus: CompilerBus, scope_id: u64 }` |
| `src/evaluate/module.rs` (ModuleSystem) | **替换为** `module_events` Subject 订阅 |
| `src/sass_value/*.rs` (Value 类型) | **不变**——纯数据类型 |
| `src/cassowary/` (选择器逻辑) | **不变**——纯函数 |
| `src/selector/*.rs` | **不变**——选择器解析不需要 Observable |
| `src/parse/parser.rs` | 变为 `scan`-based Observable |
| `src/parse/lexer.rs` | 变为 `Observable::from_iter(input).map(Token::from_char)` |

> 以上设计为最终方案。无进一步架构决策。/ This is the final plan. No further architecture decisions needed.
