## ADDED Requirements

### Requirement: 求值器模块禁止 for+push 命令式累积

在 `src/eval/` 模块的所有生产代码中，MUST NOT 出现 `let mut result = Vec::new(); for ... { result.push(...) }` 模式。集合变换 MUST 使用 `map`/`filter`/`collect`/`fold`/`try_fold`/`partition`/`flat_map` 等迭代器方法。

#### Scenario: builtin merge_args 使用 iterator chain

- **WHEN** `merge_args`、`merge_meta_args`、`merge_color_args`、`merge_mix_args` 函数合并位置参数和关键字参数
- **THEN** 使用 `param_names.iter().enumerate().map(|(i, pname)| pos_args.get(i).or_else(|| kw_args.get(*pname)).cloned()).collect()` 模式，而非 `for (i, pname) in param_names.iter().enumerate() { if i < pos_args.len() { result.push(pos_args[i].clone()) } }`

#### Scenario: exit_scope 传播使用 into_iter

- **WHEN** `Env::exit_scope` 需要将规则体内的变更传播回 saved scope
- **THEN** 使用 `rule_local_vars.into_iter().filter(|(name, _)| name.contains('.')).for_each(|(name, val)| { self.local_vars.insert(name, val); })` 模式，而非 `for (name, val) in &rule_local_vars { if name.contains('.') { self.local_vars.insert(name.clone(), val.clone()) } }`

#### Scenario: extend apply_extends 使用 partition

- **WHEN** `apply_extends` 需要分流占位符选择器和普通选择器
- **THEN** 使用 `partition` 或 `filter` 迭代器方法，而非 `for part in &parts { if part == target { new_selectors.push(...) } }`

### Requirement: CSS 模块禁止 for+push 命令式累积

在 `src/css/` 模块的所有生产代码中，MUST NOT 出现 `for ... { result.push(...) }` 模式。

#### Scenario: flatten_nodes 使用迭代器链

- **WHEN** `flatten_nodes` 展平 CSS 节点树
- **THEN** 使用 `scan` + `flat_map` 迭代器链，而非 `for node in nodes { match node { ... result.push(...) } }`

#### Scenario: serialize_expanded 使用 fold

- **WHEN** `serialize_expanded` 序列化 CSS 节点为字符串
- **THEN** 使用 `fold` 或 `flat_map` 累积输出，而非 `for node in nodes { ... result.push_str(...) }`

### Requirement: 内建函数模块禁止 for+push

在 `src/eval/builtin/` 目录下所有文件中，MUST NOT 出现 `for+push` 模式。参数合并 MUST 使用 `enumerate().map().collect()` 模式。

#### Scenario: 参数合并函数使用 iterator

- **WHEN** `merge_args`、`merge_meta_args`、`merge_color_args`、`merge_mix_args` 合并参数
- **THEN** 使用 `param_names.iter().enumerate().map(...).collect()` 模式

#### Scenario: list 函数使用 iterator

- **WHEN** `list` 内建函数处理列表元素
- **THEN** 使用 `items.iter().enumerate().map(...)` 或 `fold` 模式，而非 `for (i, item) in items.iter().enumerate() { ... }`
