## ADDED Requirements

### Requirement: Env 不可变约束

`Env` 的所有字段修改 MUST 通过 builder 方法（`self -> Self`）完成。`eval/` 内部代码 MUST NOT 直接对 `Env` 实例的字段赋值（如 `env.depth = 1`），MUST 使用对应的 builder 方法（如 `env.with_depth(1)`）。

#### Scenario: Env 字段修改通过 builder 方法

- **WHEN** 求值器需要修改 Env 的 depth 字段
- **THEN** 代码调用 `env.with_depth(new_depth)` 而非 `env.depth = new_depth`

#### Scenario: Env 字段修改通过 builder 方法（plain_css）

- **WHEN** 求值器需要设置 plain_css 模式
- **THEN** 代码调用 `env.with_plain_css(true)` 而非 `env.plain_css = true`

#### Scenario: 跨文件不可直接访问 Env 字段

- **WHEN** 任何 `eval/` 子模块需要修改 Env 状态
- **THEN** 通过 `Env` 的 `pub(crate)` builder 方法完成，不直接操作内部字段

### Requirement: eval_node 纯函数分发

`eval_node` 函数 MUST 仅做 match 分发，每个 match arm MUST 委托给一个独立的函数。match arm 内 MUST NOT 包含求值逻辑（如 `eval_value` 调用、`CssNode` 构造、`env` 状态修改）。

#### Scenario: 每个 Node 变体有独立函数

- **WHEN** `eval_node` 处理 `Node::Decl` 变体
- **THEN** 委托给 `eval_decl(property, value, important, env)` 函数，match arm 不含内联逻辑

#### Scenario: 每个 Node 变体有独立函数（Comment）

- **WHEN** `eval_node` 处理 `Node::Comment` 变体
- **THEN** 委托给 `eval_comment(text, silent, env)` 函数

### Requirement: 后处理纯函数化

`apply_extends` 和 `hoist_css_imports` MUST 消费 `Vec<CssNode>` 并返回新的 `Vec<CssNode>`，MUST NOT 使用 `&mut` 参数就地修改。

#### Scenario: apply_extends 返回新 Vec

- **WHEN** 求值完成后有 extends 需要应用
- **THEN** 调用 `let css = apply_extends(css, &extends)` 而非 `apply_extends(&mut css, &extends)`

#### Scenario: hoist_css_imports 返回新 Vec

- **WHEN** CSS 树中有 @import 需要提升到顶部
- **THEN** 调用 `let css = hoist_css_imports(css)` 而非 `hoist_css_imports(&mut css)`

### Requirement: env.clone 限制

`env.clone()` MUST NOT 出现在除 `@content` 上下文快照以外的任何代码路径中。`bind_params`、`call_user_function`、`call_module_function` MUST 使用 move 语义接收 `Env`。

#### Scenario: bind_params 不 clone Env

- **WHEN** `exec_mixin` 调用 `bind_params` 绑定参数
- **THEN** `bind_params` 接收 `&Env` 或 move `Env`，不调用 `env.clone()`

#### Scenario: @content 上下文允许 clone

- **WHEN** mixin 执行时需要保存 `@content` 块的调用者环境
- **THEN** 允许 `env.clone()` 作为上下文快照（这是唯一例外）

### Requirement: 文件按功能域组织

源文件 MUST 按功能域命名和组织。一个文件内的所有函数 MUST 属于同一功能域。MUST NOT 为了满足行数限制而将不相关的函数放在同一文件中。

#### Scenario: mixin.rs 只包含 mixin 相关

- **WHEN** 检查 `mixin.rs` 的内容
- **THEN** 文件只包含 `eval_include`、`exec_mixin`、`bind_params` 等 mixin 相关函数，不包含 `call_function`、`eval_at_rule` 等不相关函数

#### Scenario: 文件可超过 500 行

- **WHEN** 一个功能域的代码超过 500 行但全部属于同一功能
- **THEN** 不强制拆分，以功能内聚度为优先标准
