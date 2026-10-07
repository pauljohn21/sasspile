# Spec Delta

## MODIFIED Requirements

### Requirement: System SHALL provide a `SassOp` trait for all statement types / 系统 SHALL 为所有语句类型提供 `SassOp` trait

系统 SHALL 定义 `pub trait SassOp`，包含方法 `fn into_operator(self, ctx: Rc<EvalContext>) -> Observable<AstNode>`。每个代表 Sass 语句的 AST 节点类型 SHALL 实现此特征。`self` 被移动到（move）`Observable::create` 闭包中，强制单所有者语义。操作符 SHALL 通过 `Observable::create` 构建，直接接入 rxrust 调度、背压、取消系统。**MODIFIED**: 原规定返回 `Box<dyn Fn(Observable<AstNode>) -> Observable<AstNode>>` 函数指针，改为直接返回 `Observable<AstNode>` 原生算子，以完整接入 rxrust 调度/背压/取消系统。

The system SHALL define `pub trait SassOp` with method `fn into_operator(self, ctx: Rc<EvalContext>) -> Observable<AstNode>`. Each AST node type SHALL implement this trait. `self` is moved into the `Observable::create` closure. The operator SHALL use `Observable::create` to directly integrate with rxrust scheduling/backpressure/cancellation. **MODIFIED**: Original specification returned `Box<dyn Fn(Observable<AstNode>) -> Observable<AstNode>>` function pointer, changed to return `Observable<AstNode>` native operator for full rxrust integration.

#### Scenario: AstIf implements SassOp / AstIf 实现 SassOp
- **WHEN / 当** `AstIf { clauses: [...], else_clause: [...] }` 调用 `into_operator(ctx)` 时
- **THEN / 那么** 返回 `Observable::create` 构建的算子，通过 `switch_map` 在变量环境上选择分支

#### Scenario: AstFor implements SassOp / AstFor 实现 SassOp
- **WHEN / 当** `AstFor { var: "$i", from: 1, through: 10, body: [...] }` 调用 `into_operator(ctx)` 时
- **THEN / 那么** 返回 `Observable::create` 构建的算子，通过 `flat_map` 在范围内迭代，每次迭代求值一次

#### Scenario: Operator cancellation propagates through native operators / 算子取消通过原生算子传播
- **WHEN / 当** 调用方取消订阅 `@for` 算子输出时
- **THEN / 那么** 内部 `@if` 算子的 `Observable::create` 闭包 SHALL 收到取消通知并停止处理

### Requirement: Operator composition SHALL preserve ownership semantics / 操作符组合 SHALL 保持所有权语义

每个 `SassOp` 实现 SHALL 将 `self`（AST 节点）移动到（move）`Observable::create` 闭包中。当嵌套操作符时，`ctx: Rc<EvalContext>` SHALL 通过 `Rc::clone` 复制（而非移动）。**MODIFIED**: 原规定操作符闭包是 `'static` 的 `Box<dyn Fn>`，改为 `Observable::create` 生成的算子——后者自动管理闭包生命周期的 `'static` 约束。

Each `SassOp` implementation SHALL move `self` into the `Observable::create` closure. The `ctx: Rc<EvalContext>` SHALL be cloned (not moved) when passed into nested operators. **MODIFIED**: Original specification used `'static` `Box<dyn Fn>` closure, changed to `Observable::create` native operator which automatically manages `'static` closure lifetime.

#### Scenario: Nested directives share context via Rc clone / 嵌套指令通过 Rc clone 共享上下文
- **WHEN / 当** 编译 `@for $i in 1 through 3 { @if $i > 1 { .w-#{$i} { ... } } }` 时
- **THEN / 那么** 内层 `@if` 接收外层 `@for` 上下文的 `Rc::clone`
- **THEN / 那么** 两个操作符持有指向同一 `EvalContext` 的独立 `Rc` 指针

#### Scenario: Operator cancellation propagates through native operators / 算子取消通过原生算子传播
- **WHEN / 当** 调用方取消订阅 `@for` 算子输出时
- **THEN / 那么** 内部 `@if` 算子的 `Observable::create` 闭包 SHALL 收到取消通知并停止处理
