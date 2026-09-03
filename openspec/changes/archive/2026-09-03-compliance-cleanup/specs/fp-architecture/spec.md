## MODIFIED Requirements

### Requirement: 禁止 Vec::new + for push 模式

在 `src/` 目录的所有生产代码中，MUST NOT 出现 `let mut result = Vec::new(); for ... { result.push(...) }` 模式。循环累积 MUST 使用 `fold`、`collect`、`flat_map` 或 `try_fold` 替代。

例外：解析器（`parse/`）和词法分析器（`lex/`）的有状态状态机循环 MAY 保留 `for` 循环，但 SHOULD 避免内部 `let mut result = Vec::new()` 模式。

新增：`Env::exit_scope` 的 scope 传播循环 MUST 使用 `into_iter().filter().for_each()` 或 `fold` 替代 `for (name, val) in &map { self.insert(name.clone(), val.clone()) }` 模式。涉及 6 个 HashMap 的 scope 传播循环 MUST 引入 `ScopeSnapshot` 结构体通过 move 语义传递，而非 clone 深拷贝。

#### Scenario: 求值器模块禁止 Vec push 循环

- **WHEN** 检查 `eval/` 模块内的任何函数
- **THEN** 不存在 `let mut result = Vec::new(); for ... { result.push(...) }` 模式，改用 `fold` / `collect` / `flat_map`

#### Scenario: CSS 模块禁止 Vec push 循环

- **WHEN** 检查 `css/` 模块内的任何函数
- **THEN** 不存在 `let mut result = Vec::new(); for ... { result.push(...) }` 模式，改用 `fold` / `collect` / `flat_map`

#### Scenario: 解析器保留 for 循环但消除内部 Vec push

- **WHEN** 检查 `parse/` 模块内的函数
- **THEN** `while let Some(t) = self.peek()` 状态机循环 MAY 保留，但内部 `let mut items = Vec::new(); items.push(...)` SHOULD 改为收集后再 `collect`

#### Scenario: exit_scope scope 传播使用 move 语义

- **WHEN** `Env::exit_scope` 需要将规则体内产生的变更传播回 saved scope
- **THEN** 使用 `ScopeSnapshot` 结构体通过 move 语义传递 6 个 HashMap，而非 `env.get_local_vars().clone()` × 6 深拷贝。传播循环使用 `into_iter().filter().for_each()` 而非 `for (name, val) in &map { self.insert(name.clone(), val.clone()) }`
