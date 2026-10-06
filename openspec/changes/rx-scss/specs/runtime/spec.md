# Spec Delta

## Purpose

`EvalContext` 提供不可变的运行时上下文（`bus: CompilerBus` + `scope_id: u64`），通过 `CompileBuilder::build` 内部调用 `create_runtime()` 函数创建。child_scope 通过 ID 算术派生，变量查找向上遍历，全函数式 API 消除 GC 风格。

## ADDED Requirements

### Requirement: EvalContext 不可变对
`EvalContext`  SHALL 是一个 `bus: CompilerBus` + `scope_id: u64` 的结构体。两个字段均私有，通过 `bus()` / `scope_id()` 只读访问。不可变语义确保多线程共享无需锁。

#### Scenario: Read bus reference
- **WHEN** `ctx.bus()` 被调用
- **THEN** SHALL 返回 &CompilerBus 不可变引用

#### Scenario: Read scope_id
- **WHEN** `ctx.scope_id()` 被调用
- **THEN** SHALL 返回 u64 scope 标识

### Requirement: Child scope 派生
`ctx.child_scope(local_idx: u64) -> Self` SHALL 生成 `scope_id = parent_id * 1000 + local_idx`。Child scope 共享相同的 CompilerBus 引用。

#### Scenario: First child scope
- **WHEN** parent scope_id=1, `ctx.child_scope(1)` 被调用
- **THEN** SHALL 返回 EvalContext { scope_id: 1001, bus: same }

#### Scenario: Grandchild scope
- **WHEN** scope_id=1001, `ctx.child_scope(2)` 被调用
- **THEN** SHALL 返回 scope_id=1001002

### Requirement: 变量读取
`ctx.var(name: &str) -> Option<Value>` SHALL 委托 `self.bus.get_var(self.scope_id, name)`。查找 SHALL 先精确匹配 scope，未找到则向上遍历祖先 scope。

#### Scenario: Read own scope variable
- **WHEN** scope_id=1 绑定了 "x"，`ctx.var("x")` 被调用
- **THEN** SHALL 返回 Some(绑定的值)

#### Scenario: Read parent scope variable
- **WHEN** scope_id=1 绑定了 "x"，scope_id=1001 的 ctx 调用 `var("x")`
- **THEN** SHALL 继承自父 scope，返回 Some(值)

#### Scenario: Variable not found
- **WHEN** 在任何作用域中都不存在 "missing"
- **THEN** SHALL 返回 None

### Requirement: 变量绑定
`ctx.bind_var(name: String, value: Value)` SHALL 调用 `self.bus.set_var(self.scope_id, name, value)`。绑定 SHALL 广播到 `var_events` 通道。

#### Scenario: Bind and read back
- **WHEN** `ctx.bind_var("color".into(), Value::String("red".into()))` 后调用 `ctx.var("color")`
- **THEN** SHALL 返回 Some(Value::String("red"))

### Requirement: create_runtime 函数
`fn create_runtime() -> (Arc<EvalContext>, Arc<CompilerBus>)` SHALL 定义为 `runtime` 模块的公共函数，创建新的 CompilerBus 和 root scope (scope_id=1) 的 EvalContext。两者都被 Arc 包装以支持多线程共享。

#### Scenario: Default runtime creation
- **WHEN** `create_runtime()` 被调用
- **THEN** SHALL 返回 (Arc<EvalContext { scope_id: 1 }>, Arc<CompilerBus>)

### Requirement: Clone 语义
`EvalContext` SHALL derive `Clone`。`clone()` SHALL 返回新的 struct 实例但指向相同的 Arc<CompilerBus>。不产生新的 scope_id。

#### Scenario: Context clone shares bus
- **WHEN** cloned ctx 上 `bind_var` 后通过原 ctx 调用 `var`
- **THEN** SHALL 读到绑定的值（因为共享同一 Bus）

### Requirement: Send + Sync
`EvalContext` SHALL 满足 `Send + 'static`。CompilerBus 的类型保证 `EvalContext` 天然满足这两条约束。

#### Scenario: Context moved across threads
- **WHEN** `std::thread::spawn(move || ctx.var("x"))` 被调用
- **THEN** 编译 SHALL 成功

