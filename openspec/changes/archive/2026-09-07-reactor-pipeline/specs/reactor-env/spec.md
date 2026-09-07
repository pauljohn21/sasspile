# Spec: Reactor Env (持久化作用域)

## ADDED Requirements

### Requirement: Env 消费-返回 API
ENV SHALL 提供 `&self` (只读查询) 或 `self -> Self` (绑定操作) 方法, 禁止 `&mut self` 和 `&mut Scope`。

#### Scenario: bind 返回新 Env
- **WHEN** 调用 `env.bind("x", Value::Number(1.0, None))`
- **THEN** 返回新 `Env`, 旧 `env` 依然可用且不含 `x`

#### Scenario: 嵌套作用域查找
- **WHEN** 在一个已绑定外层变量的作用域中进入新作用域, 查找该变量
- **THEN** 沿 `parent` 链找到外层变量, 返回 `Some(&Value)`

### Requirement: Scope 持久化数据结构
SCOPE SHALL 使用 `imbl::HashMap` 存储 vars/mixins/functions, clone 操作共享底层结构。

#### Scenario: clone 后 parent 链共享
- **WHEN** 执行 `let env2 = env1.enter_scope()`
- **THEN** `env2.scope.parent` 与 `env1.scope` 共享 (Rc::clone), 无深层拷贝

### Requirement: 保持现有 Sass 作用域语义
ENV 改造 SHALL 保持现有 SCSS 作用域语义 (flow-control @if/@for/@each 不创建新作用域)。

#### Scenario: @if 内绑定的变量对外可见
- **WHEN** `@if $cond { $x = 1; }` 求值完成后
- **THEN** 外部作用域能读取 `$x = 1`

### Requirement: 作用域进出与 Reactor 集成
REACTOR SHALL 通过消费-返回方式传递 Env, 不通过 `&mut env` 修改。

#### Scenario: eval_block 返回更新后的 Env
- **WHEN** 调用 `eval_block(reactor, block)`
- **THEN** 返回 `(Reactor, Vec<CssNode>)`, 新 Reactor 包含更新后的 env
- **AND** 同一 block 在不同 env 中求值不产生 interference
