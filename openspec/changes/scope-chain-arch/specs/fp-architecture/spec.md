## MODIFIED Requirements

### Requirement: 禁止 Vec::new + for push 模式

在 `src/` 目录的所有生产代码中，MUST NOT 出现 `let mut result = Vec::new(); for ... { result.push(...) }` 模式。循环累积 MUST 使用 `fold`、`collect`、`flat_map` 或 `try_fold` 替代。

作用域进出 MUST 使用 `enter_scope()` / `exit_scope()` 创建和恢复 `Scope`，MUST NOT 通过 `clone` 整个 `Env` 或多个 HashMap 来保存作用域快照。`Rc::clone`（原子计数器递增）不视为 clone 操作。

例外：解析器（`parse/`）和词法分析器（`lex/`）的有状态状态机循环 MAY 保留 `for` 循环，但 SHOULD 避免内部 `let mut result = Vec::new()` 模式。

`@content` 上下文快照 MAY 保留 `Rc<Scope>` 指针复制（非 HashMap clone）。

#### Scenario: 求值器模块禁止 Vec push 循环

- **WHEN** 检查 `eval/` 模块内的任何函数
- **THEN** 不存在 `let mut result = Vec::new(); for ... { result.push(...) }` 模式，改用 `fold` / `collect` / `flat_map`

#### Scenario: CSS 模块禁止 Vec push 循环

- **WHEN** 检查 `css/` 模块内的任何函数
- **THEN** 不存在 `let mut result = Vec::new(); for ... { result.push(...) }` 模式，改用 `fold` / `collect` / `flat_map`

#### Scenario: 解析器保留 for 循环但消除内部 Vec push

- **WHEN** 检查 `parse/` 模块内的函数
- **THEN** `while let Some(t) = self.peek()` 状态机循环 MAY 保留，但内部 `let mut items = Vec::new(); items.push(...)` SHOULD 改为收集后再 `collect`

#### Scenario: 作用域进出禁止 HashMap clone

- **WHEN** 进入或退出规则体、mixin 调用、function 调用作用域
- **THEN** 使用 `enter_scope()` / `exit_scope()` 管理作用域，不调用 `HashMap::clone()` 保存快照

#### Scenario: @content 快照允许 Rc 指针复制

- **WHEN** mixin 调用方传递 `@content` 块
- **THEN** 快照通过 `Rc<Scope>` 指针复制实现，不 clone 整个 `Env` 或 HashMap
