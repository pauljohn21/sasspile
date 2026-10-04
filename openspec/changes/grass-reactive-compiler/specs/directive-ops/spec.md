# Spec Delta

## Purpose

定义基于特征（trait-based）的算子系统（operator system）。每个 Sass `@`-指令和语句类型都是独立的 `SassOp` 实现，通过变换 Observable 流来完成工作。这使得每条指令的生命周期完全隔离，不与任何其他指令共享可变状态。可扩展（extensible）：添加一条新指令只需文件级别的修改，不需要触碰现有代码。

Defines the trait-based operator system where each Sass `@`-directive and statement type is an independent `SassOp` implementation that transforms Observables, enabling composable, extensible directive semantics where each directive's lifecycle is isolated and doesn't share mutable state with other directives.

## ADDED Requirements

### Requirement: System SHALL provide a `SassOp` trait for all statement types / 系统 SHALL 为所有语句类型提供 `SassOp` trait

系统 SHALL 定义 `pub trait SassOp`，包含方法 `fn into_operator(self, ctx: Rc<EvalContext>) -> Box<dyn Fn(Observable<AstNode>) -> Observable<AstNode>>`。每个代表 Sass 语句的 AST 节点类型 SHALL 实现此特征。`self` 被移动到（move）操作符闭包中，强制单所有者语义。

The system SHALL define `pub trait SassOp` with method `fn into_operator(self, ctx: Rc<EvalContext>) -> Box<dyn Fn(Observable<AstNode>) -> Observable<AstNode>>>`. Each AST node type that represents a Sass statement SHALL implement this trait. `self` is moved into the operator closure, enforcing single-owner semantics.

> **设计意图 / Design intent**: 操作符的签名是 `AstNode → AstNode`（不是 AstNode → CssStmt），因为 `@`-指令可能需要多次 AST 展开步骤（例如 `@for` 展开为 N 个身体节点的副本）。

#### Scenario: AstIf implements SassOp / AstIf 实现 SassOp
- **WHEN / 当** `AstIf { clauses: [...], else_clause: [...] }` 调用 `into_operator(ctx)` 时
- **THEN / 那么** 返回一个闭包，通过 `switch_map` 在变量环境上选择分支，映射 `Observable<AstNode>` → `Observable<AstNode>`

#### Scenario: AstFor implements SassOp / AstFor 实现 SassOp
- **WHEN / 当** `AstFor { var: "$i", from: 1, through: 10, body: [...] }` 调用 `into_operator(ctx)` 时
- **THEN / 那么** 返回一个闭包，通过 `flat_map` 在 `Local::from_iter(range)` 上迭代，每次迭代求值一次

### Requirement: @if directive SHALL use switch_map to select exactly one branch / @if 指令 SHALL 使用 switch_map 选择一个分支

`AstIf` 的 `SassOp` 实现 SHALL 使用 `switch_map` 操作符来评估 clause 条件，并为求值选择恰好一个分支的 body。条件按顺序求值；第一个真值条件的 body 被使用。

The `SassOp` implementation for `AstIf` SHALL use `switch_map` operator to evaluate clause conditions and select exactly one branch's body for evaluation.

> **为什么用 switch_map / Why switch_map**: switch_map 有一个关键语义——当新的上游项到达时，之前内部 Observable 会被自动取消（unsubscribe）。这意味着一旦某个 `@if` 条件为真，后续条件的求值会被立即取消，保证"恰好一个分支"的语义。

#### Scenario: @if with true first clause / 第一个条件为真
- **WHEN / 当** Sass 为 `@if $x { .a { color: red; } } @else { .a { color: blue; } }` 且 `$x` 为真值时
- **THEN / 那么** 仅第一个分支（`.a { color: red; }`）被求值并发射

#### Scenario: @else if chain / @else if 链
- **WHEN / 当** Sass 为 `@if $a { ... } @else if $b { ... } @else { ... }`，其中 `$a` 为假，`$b` 为真时
- **THEN / 那么** 仅第二个分支的输出被发射

#### Scenario: @if with no matching clause falls through to @else / 无匹配时回退到 @else
- **WHEN / 当** 所有 `@if`/`@else if` 条件都为假值，且存在 `@else` 子句时
- **THEN / 那么** `@else` 的 body 被求值

### Requirement: @for directive SHALL iterate a numeric range emitting CSS per iteration / @for 指令 SHALL 迭代数值范围，每次迭代发射 CSS

`AstFor` 的 `SassOp` 实现 SHALL 从 `from` 迭代到 `to`（`through` 包含结束值，`to` 不包含），在每次迭代中绑定循环变量，并为每次迭代发射求值后的 body CSS。

The `SassOp` implementation for `AstFor` SHALL iterate from `from` to `to` (inclusive for `through`, exclusive for `to`), binding the loop variable on each iteration, and emitting the evaluated body's CSS.

#### Scenario: @for through range / through 范围
- **WHEN / 当** Sass 为 `@for $i from 1 through 3 { .w-#{$i} { width: #{$i}px; } }` 时
- **THEN / 那么** 发射 `.w-1{width:1px}`、`.w-2{width:2px}`、`.w-3{width:3px}`

#### Scenario: @for excludes end for `to` keyword / to 关键字排除结束值
- **WHEN / 当** Sass 为 `@for $i from 1 to 3 { .w-#{$i} { width: #{$i}px; } }` 时
- **THEN / 那么** 仅迭代 1、2（3 被排除）

#### Scenario: @for descending range / 降序范围
- **WHEN / 当** Sass 为 `@for $i from 3 through 1 { .w-#{$i} { width: #{$i}px; } }` 时
- **THEN / 那么** 按 3、2、1 的顺序迭代

### Requirement: @each directive SHALL iterate list or map emitting CSS per item / @each 指令 SHALL 迭代列表或映射，每个元素发射 CSS

`AstEach` 的 `SassOp` 实现 SHALL 迭代列表表达式中的每个元素，绑定迭代变量（们），并发射 body 的 CSS。多重变量时，元素会被解构。

The `SassOp` implementation for `AstEach` SHALL iterate each item in the list expression, bind the iteration variable(s), and emit the body's CSS.

#### Scenario: @each over list / 列表迭代
- **WHEN / 当** Sass 为 `@each $color in red, green, .#{$color} { color: $color }` 时
- **THEN / 那么** 每个颜色生成一条 CSS 规则

#### Scenario: @each over map with key-value destructuring / 映射迭代带键值解构
- **WHEN / 当** Sass 为 `@each $key, $value in (a: 1, b: 2) { ... }` 时
- **THEN / 那么** 每次迭代绑定 `$key` 和 `$value`

### Requirement: @while directive SHALL iterate until condition is falsy / @while 指令 SHALL 迭代直到条件为假

`AstWhile` 的 `SassOp` 实现 SHALL 循环求值条件，条件为真时持续求值 body。实现 SHALL 包含安全边界以防止无限循环（MAX_WHILE_ITERATIONS = 10000）。

The `SassOp` implementation for `AstWhile` SHALL repeatedly evaluate the body while the condition is truthy, with a safety boundary (MAX_WHILE_ITERATIONS = 10000) to prevent infinite loops.

#### Scenario: @while terminates / @while 终止
- **WHEN / 当** Sass 为 `@while $i > 0 { ... $i: $i - 1 }` 起始 `$i: 3` 时
- **THEN / 那么** body 被求值 3 次，当 `$i` 到达 0 时停止

#### Scenario: @while respects iteration limit / 尊重迭代限制
- **WHEN / 当** `@while` 循环将超过 MAX_WHILE_ITERATIONS 次迭代时
- **THEN / 那么** 循环终止并发射错误

### Requirement: @mixin SHALL store definition in registry without emitting to CSS stream / @mixin SHALL 将定义存入注册表，不发射到 CSS 流

`AstMixin` 的 `SassOp` 实现 SHALL 使用 `tap`（副作用操作符）将 mixin 定义注册到模块系统。Observable 流 SHALL 原样通过（声明不产生 CSS 输出）。

The `SassOp` implementation for `AstMixin` SHALL use `tap` (side-effect operator) to register the mixin definition into the module system.

#### Scenario: @mixin declaration produces no CSS / @mixin 声明不产生 CSS
- **WHEN / 当** Sass 为 `@mixin foo($x) { .a { width: $x } }` 时
- **THEN / 那么** CSS 流不发射任何块
- **THEN / 那么** 后续的 `@include foo(10px)` 能求值 mixin 的 body

### Requirement: @include SHALL expand mixin body with bound arguments / @include SHALL 展开 mixin 的 body 并绑定参数

`AstInclude` 的 `SassOp` 实现 SHALL 从模块注册表检索 mixin 定义，将实参绑定到形参，并通过管道求值 body。

The `SassOp` implementation for `AstInclude` SHALL retrieve the mixin definition from the module registry, bind arguments to parameters, and evaluate the body.

#### Scenario: @include emits mixin body CSS / @include 发射 mixin body CSS
- **WHEN / 当** mixin `foo($x) { .a { width: $x } }` 通过 `@include foo(20px)` 被包含时
- **THEN / 那么** 发射 `.a{width:20px}`（或展开模式等价格式）

#### Scenario: @include passes content block / @include 传递 content block
- **WHEN / 当** `@include foo { .child { color: red } }` 且 mixin 包含 `@content` 时
- **THEN / 那么** `@content` 占位符被提供的 block 输出替换

### Requirement: @function SHALL register callable and @return SHALL terminate function Observable / @function SHALL 注册可调用对象，@return SHALL 终止函数 Observable

`AstFunctionDecl` 的 `SassOp` 实现 SHALL 通过 `tap` 将函数注册到函数注册表。`AstReturn` 的实现 SHALL 终止当前函数的内部 Observable 流，将返回值发射给调用方。

The `SassOp` implementation for `AstFunctionDecl` SHALL register the function via `tap`. AstReturn SHALL terminate the current function's inner Observable.

#### Scenario: @function returns computed value / @function 返回计算值
- **WHEN / 当** 函数 `add($a, $b) { @return $a + $b }` 被作为 `add(1, 2)` 调用时
- **THEN / 那么** 调用点收到 `Value::Number(3)`

### Requirement: @use SHALL compile module and merge into environment via multicast / @use SHALL 编译模块并通过多播合并到环境

`AstUseRule` 的 `SassOp` 实现 SHALL 异步编译目标模块（如果尚未编译），然后通过多播 `module_events` Subject 将模块的导出成员合并到当前环境。

The `SassOp` implementation for `AstUseRule` SHALL asynchronously compile the target module, then merge exported members via the multicast `module_events` Subject.

#### Scenario: @use makes module members available / @use 使模块成员可用
- **WHEN / 当** `_lib.scss` 定义了 `$base-color: red` 且主文件包含 `@use "lib"` 时
- **THEN / 那么** `$base-color` 在后续语句中解析为 `red`

#### Scenario: @use with namespace prefix / 带命名空间前缀
- **WHEN / 当** Sass 为 `@use "lib" as l` 时
- **THEN / 那么** 成员通过 `l.$base-color` 访问

### Requirement: @media SHALL wrap inner CSS in media query via scope buffering / @media SHALL 通过作用域缓冲将内部 CSS 包装在媒体查询中

`AstMediaRule` 的 `SassOp` 实现 SHALL 通过 `css_scope_subject` Subject 的作用域事件，收集该作用域内的所有 CSS 语句，然后作为单条 `CssStmt::Media { condition, body }` 发射。

The `SassOp` implementation for `AstMediaRule` SHALL collect CSS statements within the scope via `css_scope_subject`, then emit a single wrapped `CssStmt::Media`.

#### Scenario: @media wraps nested rules / @media 包装嵌套规则
- **WHEN / 当** Sass 为 `@media (min-width: 768px) { .a { color: red } .b { color: blue } }` 时
- **THEN / 那么** 发射 `CssStmt::Media { condition: "min-width:768px", body: [.a rule, .b rule] }`

### Requirement: @error SHALL raise pipeline error via Observable error channel / @error SHALL 通过 Observable 错误通道引发管道错误

`AstErrorRule` 的 `SassOp` 实现 SHALL 通过 Observable 的 `on_error` 通道发射错误消息，从而终止编译。

The `SassOp` implementation for `AstErrorRule` SHALL emit the error message through the Observable's `on_error` channel, terminating compilation.

#### Scenario: @error stops compilation / @error 停止编译
- **WHEN / 当** Sass 在任何位置包含 `@error "invalid configuration"` 时
- **THEN / 那么** CSS 流收到错误，不再发射后续块
- **THEN / 那么** 调用 `from_string` 返回 `Err(SassError)`

### Requirement: @warn and @debug SHALL use tap for side effects without altering the stream / @warn 和 @debug SHALL 使用 tap 进行副作用操作，不改变流

`AstWarnRule` 和 `AstDebugRule` 的 `SassOp` 实现 SHALL 使用 `tap` 操作符发射诊断消息，而不修改 Observable 流。

The `SassOp` implementations for `AstWarnRule` and `AstDebugRule` SHALL use the `tap` operator to emit diagnostic messages without modifying the Observable stream.

#### Scenario: @warn emits log but continues / @warn 发射日志但继续编译
- **WHEN / 当** Sass 为 `@warn "deprecated"; .a { color: red }` 时
- **THEN / 那么** 一条警告被记录
- **THEN / 那么** `.a { color: red }` 仍被正常编译为 CSS

### Requirement: Operator composition SHALL preserve ownership semantics / 操作符组合 SHALL 保持所有权语义

每个 `SassOp` 实现 SHALL 将 `self`（AST 节点）移动到（move）操作符闭包中。操作符闭包 SHALL 是 `'static`（无借用引用）。当嵌套操作符时，`ctx: Rc<EvalContext>` SHALL 通过 `Rc::clone` 复制（而非移动）。

Each `SassOp` implementation SHALL move `self` into the operator closure. The operator closure SHALL be `'static`. The `ctx: Rc<EvalContext>` SHALL be cloned (not moved) when passed into nested operators.

#### Scenario: Nested directives share context via Rc clone / 嵌套指令通过 Rc clone 共享上下文
- **WHEN / 当** 编译 `@for $i in 1 through 3 { @if $i > 1 { .w-#{$i} { ... } } }` 时
- **THEN / 那么** 内层 `@if` 接收外层 `@for` 上下文的 `Rc::clone`
- **THEN / 那么** 两个操作符持有指向同一 `EvalContext` 的独立 `Rc` 指针
