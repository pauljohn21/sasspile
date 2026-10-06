# Spec Delta

## Purpose

将 `AstNode` 流通过 `dispatch_op` 函数将每个指令转化为 rxrust 原生算子，通过 `flat_map` 组合为 `CssStmt` 流的求值阶段。每个指令是独立的 `Shared::create` 算子。`dispatch_op` 作为 `eval::dispatch` 模块中的独立函数定义。

## ADDED Requirements

### Requirement: dispatch_op 函数分发
`fn dispatch_op(node: AstNode, ctx: Arc<EvalContext>) -> AstStream` SHALL 定义为 `eval::dispatch` 公共函数，根据 `node` 变体分发到对应的指令算子。该函数被 `CompileBuilder::build` 内部通过 `flat_map(|n| dispatch_op(n, ctx.clone()))` 调用。

#### Scenario: VariableDecl dispatched to bind operator
- **WHEN** `dispatch_op` 接收 `AstNode::VariableDecl { name, value, scope_id }`
- **THEN** SHALL 调用 `ctx.bind_var(name, eval(value, ctx))` 并返回空流

#### Scenario: StyleDecl dispatched to CSS operator
- **WHEN** `dispatch_op` 接收 `AstNode::StyleDecl { property, value }`
- **THEN** SHALL 返回 `Shared::of(AstNode::Css(CssStmt::Decl { property, value }))`

### Requirement: @if/@else 指令算子
`AstNode::If { cond, then_branch, else_branch }` 对应的算子 SHALL 求值 `cond` 为布尔。条件为真 SHALL 展开 `then_branch`，否则展开 `else_branch`。分支使用 `Shared::from_iter(...).flat_map(|n| dispatch_op(n, ctx.clone()))` 递归求值。

#### Scenario: True condition
- **WHEN** `@if true { color: red; }`
- **THEN** SHALL 产出 `CssStmt::Decl { property: "color", value: "red" }`

#### Scenario: Variable condition
- **WHEN** `@if $visible { display: block; }`
- **THEN** SHALL 通过 `ctx.var("visible")` 查找变量值并转为布尔

### Requirement: @for 指令算子
`AstNode::For { var, from, to, inclusive, body }` SHALL 计算迭代范围（inclusive=true 含 `to`，false 不含）。每次迭代将 `$var` 绑定到当前数字值，递归展开 body。

#### Scenario: Inclusive range iteration
- **WHEN** `@for $i from 1 through 3 { .w-#{$i} { width: #{$i}0px; } }`
- **THEN** SHALL 产出 `.w-1`, `.w-2`, `.w-3` 三条规则

### Requirement: @each 指令算子
`AstNode::Each { vars, list, body }` SHALL 求值 `list` 为 `Value::List`。对列表每个元素，将 `vars[0]` 绑定到当前元素，递归展开 body。

#### Scenario: Single variable each
- **WHEN** `@each $color in red, green { .#{$color} { color: $color; } }`
- **THEN** SHALL 产出 `.red { color: red; }`, `.green { color: green; }`

### Requirement: @while 指令算子
`AstNode::While { cond, body }` SHALL 在每次迭代前求值 `cond`。超过 MAX_WHILE_ITERATIONS（10,000）次迭代时停止。

#### Scenario: Counter-based while
- **WHEN** `@while $i < 10 { ...; $i: $i + 1 }`
- **THEN** SHALL 迭代 10 次后停止

### Requirement: @mixin/@include 指令算子
`AstNode::MixinDecl` SHALL 注册到 `ctx.bus().register_mixin(def)` 并返回空流。`AstNode::MixinCall` SHALL 查找定义后展开 body。

#### Scenario: Mixin definition and inclusion
- **WHEN** `@mixin border-radius($r) { border-radius: $r; }` 后跟 `@include border-radius(5px);`
- **THEN** SHALL 产出 `CssStmt::Decl { property: "border-radius", value: "5px" }`

### Requirement: @function/@return 指令算子
`AstNode::FunctionDecl` SHALL 注册到 `ctx.bus().register_fn(def)`。`AstNode::Return` SHALL 求值 value 并作为函数调用结果。

#### Scenario: Function definition and call
- **WHEN** `@function double($n) { @return $n * 2; }` 后使用 `width: double(20px)`
- **THEN** double(20) SHALL 返回 `Number(40.0)`

### Requirement: @media/@supports 嵌套算子
`AstNode::Media/Supports` SHALL 在 child_scope 内递归求值 inner，收集后包装为 `CssStmt::Media/Supports { query, inner }`。

#### Scenario: Nested @media with selectors
- **WHEN** `@media screen { .a { color: red; } .b { color: blue; } }`
- **THEN** SHALL 产出 `CssStmt::Media { query: "screen", inner: [Rule(.a), Rule(.b)] }`

### Requirement: @warn/@debug 指令算子
`AstNode::Warn` SHALL 发出 `tracing::warn!` 并返回空流。`AstNode::Debug` SHALL 发出 `tracing::debug!` 并返回空流。

#### Scenario: Warn directive
- **WHEN** `@warn "deprecated";`
- **THEN** SHALL 在 tracing warn 级别输出且不产生 CSS

### Requirement: Observable 调度
每个指令算子通过 `Shared::create` 构建，天然支持 `observe_on(SharedScheduler::ThreadPool)` 跨线程调度。

#### Scenario: Parallel rule evaluation
- **WHEN** 管线配置为 `eval_stream.observe_on(Shared::pool(4))`
- **THEN** 独立规则 SHALL 在不同线程上并行求值

### Requirement: 作用域隔离
每次 @-rule 创建的子作用域 SHALL 通过 `ctx.child_scope(local_idx)` 生成唯一 scope_id。子作用域变量对父作用域不可见，但能读取父作用域变量。

#### Scenario: Parent variable access
- **WHEN** `$x: outer; @if true { @debug $x; }`
- **THEN** @debug SHALL 读到 "outer"

