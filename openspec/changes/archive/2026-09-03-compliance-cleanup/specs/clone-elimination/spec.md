## ADDED Requirements

### Requirement: Env scope 快照使用 move 语义

`Env::exit_scope` 的 scope 保存/恢复 MUST NOT 使用 `clone()` 深拷贝 6 个 HashMap。MUST 引入 `ScopeSnapshot` 结构体，通过 move 语义传递 scope 数据。

#### Scenario: eval_rule 不 clone scope 快照

- **WHEN** `eval_rule` 进入规则体执行前需要保存当前 scope
- **THEN** 调用 `env.enter_rule_scope()` 返回 `(Env, ScopeSnapshot)`，ScopeSnapshot 通过 move 持有 6 个 HashMap 的所有权，而非 `env.get_local_vars().clone()` × 6

#### Scenario: exit_rule_scope 合并变更不 clone

- **WHEN** 规则体执行完毕，需要将变更传播回 saved scope
- **THEN** `env.exit_rule_scope(snapshot)` 通过 `into_iter()` 消费 rule_local_vars 等 HashMap，而非 `for (name, val) in &rule_local_vars { self.local_vars.insert(name.clone(), val.clone()) }`

### Requirement: merge_args 系列函数不 clone 参数

`merge_args`、`merge_meta_args`、`merge_color_args`、`merge_mix_args` MUST NOT 在循环中使用 `pos_args[i].clone()`。MUST 使用 `pos_args.get(i).cloned()` 或直接 move。

#### Scenario: merge_args 使用 get+cloned 替代索引 clone

- **WHEN** `merge_args` 合并位置参数和关键字参数
- **THEN** 使用 `param_names.iter().enumerate().map(|(i, _)| pos_args.get(i).or_else(|| kw_args.get(pname)).cloned()).collect()` 模式，而非 `result.push(pos_args[i].clone())`

### Requirement: meta_ops Value 传递使用 move 或引用

`src/eval/meta_ops.rs` 中的 Value 操作 MUST NOT 频繁 `clone()`。MUST 优先使用 move 语义（消费值）或 `&Value` 不可变引用。

#### Scenario: meta_lookup 返回值不 clone

- **WHEN** `meta_lookup` 函数从模块导出中查找变量
- **THEN** 使用 `exports.local_vars.get(name).cloned()` 或 move 返回，而非 `data.clone()` + `mixin_ref.params.clone()` + `mixin_ref.body.clone()` 多次 clone

#### Scenario: meta_module_functions 不 clone ModuleExports

- **WHEN** `meta_module_functions` 遍历模块导出的函数列表
- **THEN** 使用 `&exports` 引用迭代，而非 `exports.clone()` 深拷贝整个 ModuleExports

### Requirement: @content 上下文快照保留 clone 例外

`@content` 指令的上下文快照 MAY 保留 `clone()`，因为 mixin 调用需要多次执行 `@content` 块，无法使用 move 语义。

#### Scenario: @content 快照保留 clone

- **WHEN** mixin 包含 `@content` 指令且被多次调用
- **THEN** 上下文 Env MAY 使用 `clone()` 快照，这是规则允许的例外
