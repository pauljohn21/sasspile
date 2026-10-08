# Spec Delta

## Purpose

`CompilerBus` 多播事件总线通过 `SharedSubject` 实现变量绑定、模块加载、CSS 作用域开闭事件的多播分发；`Arc<Mutex<HashMap>>` 提供线程安全的 Mixin/Function/Module 注册表存储。通过 `Arc<CompilerBus>::default()` 或 `CompileBuilder::new().bus(custom_bus)` 注入。

## ADDED Requirements

### Requirement: SharedSubject 多播通道
CompilerBus SHALL 维护三个 `SharedSubject` 通道：(1) `var_events: SharedSubject<VarEvent>` — 变量绑定/更新事件；(2) `module_events: SharedSubject<ModuleEvent>` — 模块加载/成员注册事件；(3) `scope_events: SharedSubject<ScopeEvent>` — CSS 作用域开闭事件。每个通道 SHALL 是一个新的 SharedSubject 实例，支持多订阅者。

#### Scenario: Multiple subscribers receive same VarEvent
- **WHEN** 两个订阅者分别订阅 `bus.var_events()`，然后 `bus.set_var(scope, name, value)` 被调用
- **THEN** 两个订阅者 SHALL 收到相同的事件

#### Scenario: New subscription does not receive historical events
- **WHEN** 在 `set_var` 调用之后订阅 `bus.var_events()`
- **THEN** 该订阅者 SHALL 仅收到订阅之后的新事件

### Requirement: 变量注册表线程安全
CompilerBus SHALL 提供 `set_var(scope_id, name, value)` 和 `get_var(scope_id, name)` 方法。变量绑定 SHALL 通过 `Arc<Mutex<HashMap>>` 实现线程安全。`set_var` SHALL 在插入后通过 `var_events.next(VarEvent::Bind)` 广播。

#### Scenario: Concurrent variable writes
- **WHEN** 多个线程同时调用 `set_var` 作用于不同变量
- **THEN** 所有变量 SHALL 被正确存储，不发生数据竞争

#### Scenario: Variable lookup by scope
- **WHEN** `get_var(scope_id, name)` 被调用
- **THEN** SHALL 首先查找精确 scope_id，若未找到则向上遍历祖先 scope（parent = scope_id / 1000）

### Requirement: 模块注册表线程安全
CompilerBus SHALL 提供 `register_module(def)` 和 `lookup_module(path)` 方法。模块通过 `path: String` 为键存储。

#### Scenario: Module registration and lookup
- **WHEN** `register_module(ModuleDef { path: "foo", ... })` 然后 `lookup_module("foo")`
- **THEN** SHALL 返回 Some(ModuleDef)

### Requirement: Mixin 注册表线程安全
CompilerBus SHALL 提供 `register_mixin(def)` 和 `lookup_mixin(name)` 方法。

#### Scenario: Mixin registration
- **WHEN** `register_mixin(MixinDef { name: "border", ... })` 然后 `lookup_mixin("border")`
- **THEN** SHALL 返回 Some(MixinDef)

### Requirement: Function 注册表线程安全
CompilerBus SHALL 提供 `register_fn(def)` 和 `lookup_fn(name)` 方法。函数和混入 SHALL 共享独立命名空间（同名 mixin 和 function 可共存）。

#### Scenario: Function registration
- **WHEN** `register_fn(FnDef { name: "double", ... })` 然后 `lookup_fn("double")`
- **THEN** SHALL 返回 Some(FnDef)

### Requirement: 事件类型
`VarEvent` 枚举 SHALL 包含 `Bind { scope_id, name, value }` 和 `Update { scope_id, name, value }`。`ModuleEvent` SHALL 包含 `Loaded { name }` 和 `MemberRegistered { module, name }`。`ScopeEvent` SHALL 包含 `Open { scope_id, kind }` 和 `Close { scope_id, kind }`。

#### Scenario: Bind vs Update distinction
- **WHEN** 对同一 (scope_id, name) 调用 set_var 两次
- **THEN** 第一次产生 VarEvent::Bind，第二次产生 VarEvent::Update

### Requirement: ScopeKind 分类
`ScopeKind` 枚举 SHALL 包含 `Rule`、`Media`、`Supports`、`Control`、`Function`、`Mixin` 六种类型。ScopeEvent 中 SHALL 携带 `kind` 字段以标识作用域类型。

#### Scenario: Rule scope event
- **WHEN** 进入 `.a { }` 规则块
- **THEN** 应当产生 ScopeEvent::Open { scope_id, kind: ScopeKind::Rule }

### Requirement: Observable 生命周期
三个 SharedSubject 的 `clone()` 方法 SHALL 允许任何消费者独立订阅。CompilerBus 的 `drop()` 应当断开所有 Subject 通道。

#### Scenario: Bus drop disconnects subjects
- **WHEN** CompilerBus 被 drop
- **THEN** 所有订阅者 SHALL 收到 complete 通知

