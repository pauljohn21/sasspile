# Spec Delta

## Purpose

定义 CompilerBus 多播系统（multicast system）。使用 rxrust 的 Subject 作为共享的 Hot Observable 进行跨阶段通信（变量绑定、模块合并、CSS 作用域边界）。CompilerBus 替代了当前 `RefCell<Environment>` 模式，通过事件驱动的变量传播实现：组件订阅数据流而非修改共享状态。

Defines the CompilerBus multicast system using rxrust Subjects as shared Hot Observables for inter-stage communication. CompilerBus replaces the current `RefCell<Environment>` pattern with event-driven variable propagation.

> **背景 / Background**: 当前 grass 使用 `Rc<RefCell<Environment>>` 贯穿所有求值器，导致两个严重问题：(1) 运行时 borrow-check panic（内部可变性）；(2) 无法支持流式输出（必须全量求值后才能知道最终变量值）。CompilerBus 通过多播事件彻底消除这两个问题。

## ADDED Requirements

### Requirement: System SHALL define CompilerBus struct with multicast Subjects / 系统 SHALL 定义包含多播 Subject 的 CompilerBus 结构体

系统 SHALL 定义 `pub struct CompilerBus`，包含每个通信通道的多播 Subject：`var_events: Subject<ValueEvent>`、`module_events: Subject<ModuleEvent>`、`css_scope_subject: Subject<ScopeEvent>`。所有 Subject 实例 SHALL 使用默认无界通道（`Subject::new()`）。

The system SHALL define `pub struct CompilerBus` containing multicast Subjects for each communication channel: `var_events: Subject<ValueEvent>`, `module_events: Subject<ModuleEvent>`, and `css_scope_subject: Subject<ScopeEvent>`.

#### Scenario: CompilerBus is created with fresh Subjects / 创建 CompilerBus 带全新 Subject
- **WHEN / 当** 调用 `CompilerBus::new()` 时
- **THEN / 那么** 三个 Subject 被创建，没有任何 observer 连接

#### Scenario: CompilerBus can be cloned for shared access / 克隆 CompilerBus 共享访问
- **WHEN / 当** 克隆 `CompilerBus` 实例时
- **THEN / 那么** 克隆产生指向相同底层多播的引用计数指针（即 `CompilerBus` 包装内部 `Rc<InnerBus>`）

### Requirement: var_events channel SHALL propagate variable bindings / var_events 通道 SHALL 传播变量绑定

`var_events: Subject<ValueEvent>` SHALL 携带 `ValueEvent::Bind { name: String, value: Value, scope_id: u64 }` 从定义处流向消费处。消费者 SHALL 通过 `var_events.filter(...).last()` 订阅。

The `var_events: Subject<ValueEvent>` SHALL carry `ValueEvent::Bind { name: String, value: Value, scope_id: u64 }` from definition sites to consumption sites.

> **关键语义 / Key semantic**: `last()` 操作符获取给定名称的最新绑定值（最内层 shadowing scope）。这是因为后来的 Binding 覆盖先来的——这正是 Sass 的作用域规则。

#### Scenario: Variable definition reaches consumer / 变量定义到达消费者
- **WHEN / 当** Evaluator 处理 `$global: 42` 并发射 `ValueEvent::Bind { name: "$global", value: 42 }` 时
- **THEN / 那么** 任何订阅了 `$global` 事件的消费者 SHALL 立即（在下一个 AstNode 求值之前）收到该绑定

#### Scenario: Consumer sees all bindings for a name in order / 消费者按序看到同名的所有绑定
- **WHEN / 当** Sass 为 `$x: 1; $x: 2; a { width: $x }` 时
- **THEN / 那么** `$x` 的消费者按顺序收到 Bind 事件：先 1，后 2
- **THEN / 那么** 最终 CSS 使用 `$x = 2`

### Requirement: Variable subscription SHALL support reactive lookup / 变量订阅 SHALL 支持响应式查找

系统 SHALL 提供 `ver(&self, name: &str) -> impl Observable<Item = Value> + '_` 方法于 `CompilerBus` 和 `EvalContext`，返回给定变量名的所有绑定的 Observable。

The system SHALL provide a method `ver(&self, name: &str) -> impl Observable<Item = Value> + '_` that returns an Observable of all bindings for the given variable name.

#### Scenario: EvalContext reads last value of a variable / EvalContext 读取变量最后值
- **WHEN / 当** 调用 `ctx.ver("$x")` 且 `$x` 已通过 `var_events` 绑定到 10 时
- **THEN / 那么** 返回的 Observable 在订阅时立即发射 `Value::Number(10)`

#### Scenario: Context reads shadowed variable / 读取被遮蔽的变量
- **WHEN / 当** 外层作用域绑定 `$x: 1` 且内层作用域绑定 `$x: 2` 时
- **THEN / 那么** 内层上下文的 `ver("$x")` 返回 `Value::Number(2)`（内层优先）
- **THEN / 那么** 内层作用域退出后，外层上下文的 `ver("$x")` 返回 `Value::Number(1)`

### Requirement: module_events SHALL propagate module compilation results / module_events SHALL 传播模块编译结果

`module_events: Subject<ModuleEvent>` SHALL 携带 `ModuleEvent::Constructed(Rc<Module>)` 事件——当模块编译完成时发射。组件 SHALL 监听此通道以惰性（lazily）解析 `@use` 和 `@forward` 依赖。

The `module_events: Subject<ModuleEvent>` SHALL carry `ModuleEvent::Constructed(Rc<Module>)` events when a module is compiled.

#### Scenario: Module event notifies dependents / 模块事件通知依赖方
- **WHEN / 当** `@use "theme"` 触发 `theme.scss` 的编译时
- **THEN / 那么** 当 theme 编译完成时，发射 `ModuleEvent::Constructed(Rc<Module>)`
- **THEN / 那么** 任何等待 "theme" 的 @use 声明 SHALL 收到该模块

### Requirement: css_scope_subject SHALL manage scope boundaries for CSS collection / css_scope_subject SHALL 管理 CSS 收集的作用域边界

`css_scope_subject: Subject<ScopeEvent>` SHALL 携带作用域进入/退出事件，Serializer 使用这些事件来收集用于包装规则（`@media`、`@supports`、`@document`）的内部 CSS。

The `css_scope_subject: Subject<ScopeEvent>` SHALL carry scope enter/exit events that the Serializer uses to collect inner CSS for wrapping rules.

#### Scenario: Scope open triggers buffer collection / 作用域开启触发缓冲收集
- **WHEN / 当** `@media (max-width: 768px) { ... }` 开始求值时
- **THEN / 那么** 在 `css_scope_subject` 上发射 `ScopeEvent::Enter`
- **THEN / 那么** 随后的 CssStmt 项被收集到作用域缓冲区

#### Scenario: Scope close flushes wrapped CSS / 作用域关闭刷新包装的 CSS
- **WHEN / 当** `@media` block body 完成时
- **THEN / 那么** 发射 `ScopeEvent::Exit`
- **THEN / 那么** 收集到的 CssStmts 作为单条 `CssStmt::Media` 发射并包装所有收集项

### Requirement: flat_map_extract_items SHALL emit individual items from Vec results / flat_map_extract_items SHALL 从 Vec 结果中发射单个项

系统 SHALL 定义 `flat_map_extract_items` 扩展组合子（extension combinator），包装 `flat_map` 并将 `Observable<Vec<T>>` 转换为 `Observable<T>`，按顺序逐个发射内部元素。这是一个一层的展开操作——嵌套的 Vec 不会递归展开。

The system SHALL define `flat_map_extract_items` extension combinator that converts `Observable<Vec<T>>` into `Observable<T>`, emitting each element individually in order.

#### Scenario: Vec result is unwrapped / Vec 结果被展开
- **WHEN / 当** 上游发射 `vec![1, 2, 3]` 时
- **THEN / 那么** 下游收到三个独立项：`1`、`2`、`3`，按顺序

#### Scenario: Empty Vec emits nothing / 空 Vec 不发射任何项
- **WHEN / 当** 上游发射 `vec![]` 时
- **THEN / 那么** 下游不收到任何项

#### Scenario: Nested Vec is not recursively flattened / 嵌套 Vec 不递归展开
- **WHEN / 当** 上游发射 `vec![vec![1, 2], vec![3]]` 时
- **THEN / 那么** 下游收到 `vec![1, 2]` 和 `vec![3]` 作为 Vec 项（仅一层展开）

### Requirement: switch_map_extract_items SHALL emit items with latest-only semantics / switch_map_extract_items SHALL 以最新语义发射项

系统 SHALL 定义 `switch_map_extract_items` 为 `switch_map` + `flat_map` 的组合，确保只有最新上游的内部项被发射——任何进行中的旧上游项会被丢弃。

The system SHALL define `switch_map_extract_items` as a composition of `switch_map` and `flat_map` ensuring only the latest upstream's items are emitted.

#### Scenario: Race condition resolves to latest / 竞争条件解析为最新值
- **WHEN / 当** 上游发射 `A（产生 [1,2,3]）` 然后在 A 完成之前发射 `B（产生 [4,5]）` 时
- **THEN / 那么** 下游仅收到 `4, 5`；来自 A 的项被丢弃

### Requirement: Variable shadowing SHALL resolve to innermost binding / 变量遮蔽 SHALL 解析到最内层的绑定

当变量在多个作用域中被绑定时，系统 SHALL 使用 `scope_id`（单调递增的 u64，带深度层级，格式：`parent_idx * 1000 + local_counter`）进行解析。

When a variable is bound in multiple scopes, the system SHALL use a `scope_id` (monotonically increasing u64 with depth hierarchy) for lookups via `var_events.filter(...).last()`.

#### Scenario: Inner scope shadows outer / 内层遮蔽外层
- **WHEN / 当** `@for` body 每次迭代绑定 `$i` 时
- **THEN / 那么** 每次迭代的 `$i` 有独立的 `scope_id`
- **THEN / 那么** body 内的引用看到的是本次迭代的 `$i`，不是前一次迭代的

### Requirement: CompilerBus SHALL provide scope isolation for pipeline stages / CompilerBus SHALL 为管道阶段提供作用域隔离

每个管道阶段（Lexer、Parser、Evaluator、Serializer）在订阅 `CompilerBus` 时必须使用独立的 scope_id。这保证了 Evaluator 发射的 `ValueEvent::Bind` 不会污染其他阶段的订阅上下文。

Each pipeline stage SHALL use an independent scope_id when subscribing to `CompilerBus`, ensuring `ValueEvent::Bind` from Evaluator does not pollute other stages.

#### Scenario: Evaluator bindings invisible to Lexer / Evaluator 的绑定对 Lexer 不可见
- **WHEN / 当** Lexer 阶段订阅了 `var_events` 时
- **THEN / 那么** Lexer 的 scope_id 确保它仅看到 Lexer 作用域之前定义的变量，不会看到当前 Evaluator 正在求值中的变量
