## Why

sasspile 管线层已实现类型状态机链式调用（`Source.lex().parse().evaluate().serialize()`），但各阶段**内部**仍广泛使用命令式循环累积器（`let mut css = Vec::new(); for ... { css.append(...) }`）。这导致代码风格断裂：管线层是函数式链式，内部却是 `mut` 循环。全面链式化可消除 `mut` 累积器，统一数据流风格，提升可读性和一致性。

## What Changes

- `eval_nodes`：将 `for node in nodes { env = new_env }` 改为 `try_fold((Vec::new(), env), |(css, env), node| { ... })`
- `eval_for`：将 `while i != stop` 改为 `(start..stop).step_by(1).try_fold(...)`
- `eval_each`：将 `for item in &items` 改为 `items.iter().try_fold((Vec::new(), env), ...)`
- `eval_while`：保留 `loop`（无界循环不适合 fold），但将内部 `mut css` 改为函数式累积
- `hoist_css_imports`：将 `for node in nodes { if is_import { imports.push } else { rest.push } }` 改为 `into_iter().partition`
- `eval_rule`：将 3 累积器 for 循环改为封装 `RuleBuilder` 状态结构 + `fold`
- `nest_rule_in_children`：同上
- `flatten_nodes`：将递归 for 改为 `flat_map` + `collect`
- `Evaluated::serialize`：将 `&self` 改为 `self`（消费所有权，与管线链式一致）

## Capabilities

### New Capabilities

- `chain-fold`: Evaluator 内部循环累积器改为 try_fold/fold 链式反应的规范

### Modified Capabilities

（无——本次变更不改变 spec 级行为，仅改变内部实现风格）

## Impact

- `src/eval/mod.rs`：`eval_nodes`、`hoist_css_imports` 重构
- `src/eval/control_flow.rs`：`eval_for`、`eval_each`、`eval_while` 重构
- `src/eval/rule.rs`：`eval_rule`、`nest_rule_in_children` 重构
- `src/css/mod.rs`：`flatten_nodes`、`merge_at_rules` 重构
- `src/stage/evaluated.rs`：`serialize` 签名改为 `self`
- `src/lib.rs`：`compile` 系列函数适配 `serialize(self)` 签名
- 测试基线：202/202 不变 + sass-spec 2828/5362 不变
