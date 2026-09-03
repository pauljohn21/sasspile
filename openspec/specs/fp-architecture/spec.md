## Purpose

定义函数式编程架构约束，禁止命令式累积器模式，统一使用 fold/collect/flat_map 等函数式方法。

## Requirements

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

### Requirement: color_adjust 使用 apply_kw helper 消除 mut 变量重复

`color_adjust.rs` 中的 `adjust_*`、`change_*`、`scale_*` 函数 MUST NOT 重复 `let mut x = ...; if let Some(v) = get(kw, key)? { x = f(x, v); }` 模式。 MUST 使用 `apply_kw` helper 函数统一处理关键字参数应用。

#### Scenario: adjust 函数使用 apply_kw

- **WHEN** `adjust_oklab` / `adjust_lch` / `adjust_lab` 等函数需要应用关键字参数
- **THEN** 调用 `apply_kw(initial, kw, key, |val, v| f(val, v))?` 而非 `let mut x = initial; if let Some(v) = get(kw, key)? { x = f(x, v); }`

#### Scenario: change 函数使用 apply_kw

- **WHEN** `change_oklab` / `change_lch` / `change_lab` 等函数需要设置关键字参数
- **THEN** 调用 `apply_kw(initial, kw, key, |_val, v| v)?` 而非 `let mut x = initial; if let Some(v) = get(kw, key)? { x = v; }`

#### Scenario: scale 函数使用 apply_kw

- **WHEN** `scale_oklab` / `scale_lch` / `scale_lab` 等函数需要缩放关键字参数
- **THEN** 调用 `apply_kw(initial, kw, key, |val, pct| scale_fn(val, pct, max))?` 而非内联 `scale_val` 闭包 + `let mut x = ...`

### Requirement: AST 序列化使用 fold 替代 for push_str

`to_scss` 方法的 `Node::If` 分支 SHALL 使用 `branches.iter().enumerate().fold(String::new(), |mut acc, (i, (cond, body))| { ...; acc })` 替代 `let mut s = String::new(); for (i, (cond, body)) in branches.iter().enumerate() { s.push_str(...) }`。

#### Scenario: to_scss If 分支使用 fold

- **WHEN** `to_scss` 序列化 `Node::If` 节点
- **THEN** 通过 `fold` 累积字符串，不使用 `let mut s = String::new(); for (i, ...) { s.push_str(...) }`

### Requirement: selector-simple-selectors 使用 fold 替代 for push

`selector-simple-selectors` 内建函数 SHALL 使用 `s.chars().fold((Vec::new(), String::new(), false), |(mut result, mut current, ...), c| { ... })` 或等效链式风格替代 `let mut result = Vec::new(); let mut current = String::new(); for c in s.chars() { ... }`。

#### Scenario: selector-simple-selectors 使用 fold

- **WHEN** `selector-simple-selectors` 解析简单选择器列表
- **THEN** 通过 `fold` 累积结果，不使用 `let mut result = Vec::new(); for c in s.chars() { ... result.push(...) }`
