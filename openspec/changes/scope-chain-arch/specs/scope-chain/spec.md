## ADDED Requirements

### Requirement: Scope 结构体管理嵌套作用域

系统 SHALL 引入 `Scope` 结构体，包含单层 `local_vars`、`local_mixins`、`local_functions`、`forwarded_vars`、`forwarded_mixins`、`forwarded_functions` 和 `global_writes` HashMap，通过 `parent: Option<Rc<Scope>>` 链接父作用域。

`Env` SHALL 持有 `current: Rc<Scope>` 作为当前活跃作用域，外加不参与作用域链的全局字段（`content`、`namespaces`、`base_path`、`load_paths` 等）。

#### Scenario: Scope 链结构

- **WHEN** 创建一个新的 `Scope`
- **THEN** 该 `Scope` 包含 7 个 HashMap（`local_vars`、`local_mixins`、`local_functions`、`forwarded_vars`、`forwarded_mixins`、`forwarded_functions`、`global_writes`）和 `parent: Option<Rc<Scope>>`

#### Scenario: Env 持有 current scope

- **WHEN** 构造一个 `Env`
- **THEN** `Env.current` 为 `Rc<Scope>` 类型，指向当前活跃作用域

#### Scenario: Env 全局字段不参与 scope 链

- **WHEN** 进入或退出作用域
- **THEN** `content`、`content_env`、`builtin_modules`、`namespaces`、`base_path`、`depth`、`extends`、`current_selector`、`load_paths`、`plain_css`、`loaded_modules`、`module_cache`、`pending_config`、`consumed_config`、`star_members`、`star_imported` 字段不受影响

### Requirement: 零 clone 作用域进出

进入规则体、mixin 调用、function 调用作用域时，系统 SHALL 通过 `enter_scope()` 创建新的 `Scope` 并设置 `parent` 指向当前 scope，MUST NOT clone 任何 HashMap。

退出作用域时，系统 SHALL 通过 `exit_scope()` 恢复父 scope，MUST NOT clone 任何 HashMap。`Rc::clone` 操作（原子计数器递增）不视为 clone。

#### Scenario: 进入 rule scope 不 clone HashMap

- **WHEN** `eval_rule` 进入规则体作用域
- **THEN** 通过 `enter_scope()` 创建新 `Scope`，不调用任何 HashMap 的 `clone()`

#### Scenario: 退出 rule scope 恢复父 scope

- **WHEN** `eval_rule` 完成规则体求值，退出作用域
- **THEN** `Env.current` 恢复为父 scope，规则体内的局部变量不传播到外层（除 `!global` 写入和新增 mixin/function）

#### Scenario: mixin 调用不 clone Env

- **WHEN** 调用用户定义 mixin
- **THEN** 通过 `enter_scope()` 创建新作用域，不 clone 整个 `Env`

#### Scenario: function 调用不 clone Env

- **WHEN** 调用用户定义 function
- **THEN** 通过 `enter_scope()` 创建新作用域，不 clone 整个 `Env`

### Requirement: 变量查找沿 scope 链向上搜索

`lookup(name)` SHALL 从 `current` scope 开始，沿 `parent` 链向上搜索，返回第一个匹配的变量值。

#### Scenario: 查找当前 scope 的变量

- **WHEN** 变量在当前 scope 的 `local_vars` 中定义
- **THEN** 返回该变量的值

#### Scenario: 查找父 scope 的变量

- **WHEN** 变量不在当前 scope 但在父 scope 中定义
- **THEN** 沿 `parent` 链向上查找，返回父 scope 中的值

#### Scenario: 变量不存在

- **WHEN** 变量在整个 scope 链中均未定义
- **THEN** 返回 `None`

### Requirement: !global 写入通过 global_writes 中转传播

`!global` 变量赋值 SHALL 在当前 scope 的 `global_writes` 表中记录，`exit_scope()` 时 SHALL 将 `global_writes` 传播到父 scope。

`global_writes` 最终通过逐层 `exit_scope` 到达 root scope，在那里合并到 `local_vars`。

#### Scenario: !global 在非 root scope 赋值

- **WHEN** 在嵌套 rule scope 内执行 `$x: value !global`
- **THEN** `value` 被记录到当前 scope 的 `global_writes`，不在当前 scope 的 `local_vars` 中

#### Scenario: exit_scope 传播 global_writes

- **WHEN** 退出包含 `global_writes` 的 scope
- **THEN** `global_writes` 中的变量被传播到父 scope 的 `global_writes`（如果父非 root）或 `local_vars`（如果父是 root）

### Requirement: flow control 不创建新作用域

`@if`/`@else if`/`@else`、`@for`、`@each`、`@while` SHALL NOT 创建新 `Scope`。这些构造在当前 scope 内直接修改变量和绑定循环变量。

#### Scenario: @if 不创建新 scope

- **WHEN** 执行 `@if` 条件分支
- **THEN** 在当前 scope 内求值，条件分支内定义的变量在分支外可见（如果分支被执行）

#### Scenario: @for 循环变量绑定到当前 scope

- **WHEN** 执行 `@for $i from 1 through 3`
- **THEN** `$i` 绑定到当前 scope 的 `local_vars`，循环结束后变量仍可见

#### Scenario: @each 循环变量绑定到当前 scope

- **WHEN** 执行 `@each $item in $list`
- **THEN** `$item` 绑定到当前 scope 的 `local_vars`

### Requirement: @content 快照保存 Rc<Scope> 指针

`@content` 上下文快照 SHALL 保存 `Rc<Scope>`（当前 scope 指针），MUST NOT clone 整个 `Env`。

执行 `@content` 时 SHALL 从快照 scope 开始查找变量，遵循 SCSS 闭包语义。

#### Scenario: @content 快照仅保存 scope 指针

- **WHEN** mixin 调用方传递 `@content` 块
- **THEN** 快照保存 `Rc<Scope>`（原子计数器递增），不 clone 任何 HashMap

#### Scenario: @content 从快照 scope 查找变量

- **WHEN** 执行 `@content` 块
- **THEN** 变量查找从快照 scope 开始，沿其 parent 链向上搜索

### Requirement: exit_scope 传播规则

退出 scope 时，系统 SHALL 执行以下传播规则：

1. `local_vars` 中名字含 `.` 的变量（命名空间变量）SHALL 传播到父 scope
2. `global_writes` SHALL 传播到父 scope（见 !global 传播规则）
3. `local_mixins` 和 `local_functions` 中新增的定义（父 scope 中不存在的）SHALL 传播到父 scope
4. `forwarded_mixins` 和 `forwarded_functions` 中新增的定义 SHALL 传播到父 scope
5. `forwarded_vars` SHALL 传播到父 scope
6. `local_vars` 中不含 `.` 的普通变量 SHALL NOT 传播

#### Scenario: 命名空间变量传播

- **WHEN** 规则体内执行 `$module.var: value`
- **THEN** `exit_scope` 将该变量传播到父 scope 的 `local_vars`

#### Scenario: 普通局部变量不传播

- **WHEN** 规则体内执行 `$x: 1`（无 `!global`）
- **THEN** `exit_scope` 不传播 `$x` 到父 scope

#### Scenario: 新增 mixin 传播

- **WHEN** 规则体内定义 mixin `@mixin foo { ... }`
- **THEN** `exit_scope` 将 `foo` 传播到父 scope 的 `local_mixins`（如果不存在）

#### Scenario: 新增 forwarded mixin 传播

- **WHEN** 规则体内 `@forward` 添加 forwarded mixin
- **THEN** `exit_scope` 将其传播到父 scope 的 `forwarded_mixins`
