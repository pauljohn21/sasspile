## Purpose

定义 Evaluator 内部循环累积器改为 try_fold/fold 链式反应的规范，统一函数式数据流风格。

## Requirements

### Requirement: eval_nodes 使用 try_fold 链式累积

`eval_nodes` SHALL 使用 `try_fold` 替代 `for` 循环累积器。fold 状态为 `(Vec<CssNode>, Env)`，每个节点求值后 `css.extend(out)` 并更新 `env`。不再使用 `let mut env = env` 模式。

扩展覆盖范围：除 `eval_nodes` 外，所有 `eval/` 模块内产生 `Vec<CssNode>` 累积的循环 SHALL 使用 `try_fold` 或 `fold`，包括 `nest_rule_in_children`、`eval_import` CSS 分支等。

#### Scenario: 多节点求值

- **WHEN** `eval_nodes` 接收 `[Node::Decl, Node::Rule, Node::Comment]` 三个节点
- **THEN** 通过 `try_fold` 依次求值每个节点，返回 `(Vec<CssNode>, Env)`，其中 `css` 包含所有节点的输出，`env` 是最后一个节点更新后的环境

#### Scenario: 求值中途出错

- **WHEN** `eval_nodes` 接收的节点中第二个节点求值返回 `Err`
- **THEN** `try_fold` 立即短路返回 `Err`，不继续求值后续节点

#### Scenario: nest_rule_in_children 使用 fold

- **WHEN** `nest_rule_in_children` 处理一组 CSS 节点
- **THEN** 通过 `into_iter().fold(...)` 累积结果，不使用 `let mut result = Vec::new(); for ... { result.push(...) }`

#### Scenario: eval_import CSS 分支使用 map + collect

- **WHEN** `eval_import` 处理多个 CSS @import URL
- **THEN** 通过 `urls.iter().map(|u| ...).collect()` 生成节点列表，不使用 `let mut nodes = Vec::new(); for ... { nodes.push(...) }`

### Requirement: eval_for 使用 range + try_fold

`eval_for` SHALL 使用 `(start..stop).step_by(step)` + `try_fold` 替代 `while i != stop` 循环。step 方向通过计算正向 range 处理：当 `start <= end` 时用 `(start..stop).step_by(1)`，当 `start > end` 时用 `(stop+1..=start).rev()`。

#### Scenario: 正向 for 循环

- **WHEN** `@for $i from 1 through 5` 被求值
- **THEN** 使用 `(1..6).step_by(1).try_fold(...)` 迭代 5 次，每次绑定 `$i` 并求值循环体

#### Scenario: 反向 for 循环

- **WHEN** `@for $i from 5 through 1` 被求值
- **THEN** 使用 `(2..=5).rev().try_fold(...)` 迭代 5 次（1,2,3,4,5 的反向），每次绑定 `$i` 并求值循环体

### Requirement: eval_each 使用 try_fold

`eval_each` SHALL 使用 `items.iter().try_fold((Vec::new(), env), |(css, env), item| { ... })` 替代 `for item in &items` 循环。

#### Scenario: 遍历列表

- **WHEN** `@each $i in (a, b, c)` 被求值
- **THEN** 使用 `try_fold` 遍历 3 个元素，每次绑定 `$i` 并求值循环体

### Requirement: hoist_css_imports 使用 partition

`hoist_css_imports` SHALL 使用 `into_iter().partition` 替代 `for` 循环手动分流 `imports` 和 `rest`。递归嵌套处理通过前置 `map` 完成。

#### Scenario: 混合 import 和非 import 节点

- **WHEN** 输入包含 `[Rule, @import, AtRule, @import]`
- **THEN** `partition` 将 `@import` 节点分到 `imports`，其余分到 `rest`，最终 `imports ++ rest` 返回

#### Scenario: 嵌套节点先递归处理

- **WHEN** 输入包含 `AtRule { children: [@import, Rule] }`
- **THEN** 先递归处理 children（将内部 `@import` 提升），再 partition 外层

### Requirement: flatten_nodes 使用 flat_map

`flatten_nodes` SHALL 使用 `iter().flat_map(|n| Self::flatten_node(n)).collect()` 替代递归 `for` 循环。每个节点展开为零或多个扁平节点。

扩展覆盖范围：`flatten_children` 也 SHALL 使用 `flat_map` 替代 `for + push/extend` 模式。

#### Scenario: 嵌套规则展平

- **WHEN** 输入 `Rule { children: [Rule, Declaration, Rule] }`
- **THEN** `flat_map` 将嵌套 Rule 展开到顶层，返回扁平化的 `Vec<CssNode>`

#### Scenario: flatten_children 使用 flat_map

- **WHEN** `flatten_children` 处理子节点列表
- **THEN** 通过 `children.iter().flat_map(|child| { ... }).collect()` 生成扁平结果，不使用 `let mut result = Vec::new(); for child in children { ... }`

### Requirement: Evaluated::serialize 消费 self

`Evaluated::serialize` SHALL 接收 `self`（消费所有权）而非 `&self`。签名变为 `pub fn serialize(self, style: OutputStyle) -> Serialized`。

#### Scenario: 链式调用

- **WHEN** `Source::new(text).lex()?.parse()?.evaluate()?.serialize(style).into_string()` 被调用
- **THEN** `serialize` 消费 `Evaluated`，返回 `Serialized`，管线完全 owned 链式

#### Scenario: 不可重复序列化

- **WHEN** 调用 `evaluated.serialize(style)` 后再次尝试访问 `evaluated`
- **THEN** 编译错误——`evaluated` 已被 move，不可再次访问

### Requirement: eval_rule 使用 RuleBuilder + fold

`eval_rule` SHALL 封装 `struct RuleBuilder { result, current_decls, root_nodes }` 状态结构，实现 `push(node)` 方法，然后用 `css.into_iter().fold(RuleBuilder::new(), |b, n| b.push(n))` 替代 3 累积器 for 循环。

#### Scenario: 声明穿插嵌套规则

- **WHEN** 规则体产生 `[Decl, Rule, Decl, Rule, Decl]`
- **THEN** `RuleBuilder::push` 正确穿插声明和嵌套规则：声明累积到 `current_decls`，遇到 Rule 时先 flush `current_decls` 再处理嵌套规则

### Requirement: eval_while 保留 loop 但消除 mut css

`eval_while` SHALL 保留 `loop` 结构（无界循环），但消除 `let mut css = Vec::new()` 累积器，改用每次迭代的 `(out, new_env)` 直接传递。

#### Scenario: 条件求值为假时退出

- **WHEN** `@while $cond` 中 `$cond` 求值为 `false`
- **THEN** 立即退出 loop，返回累积的 `(css, env)`

#### Scenario: 循环超过限制时报错

- **WHEN** `@while` 循环迭代次数超过 `MAX_DEPTH`
- **THEN** 返回 `Err(SassError::Eval("..."))`，不继续循环

### Requirement: CSS 序列化器使用 fold 替代 for 循环

`serialize_expanded` 和 `serialize_compressed` 的外层循环 SHALL 使用 `iter().enumerate().fold(String::new(), ...)` 替代 `for (i, n) in nodes.iter().enumerate()` + `let mut result = String::new()`。内部的 `write_node_expanded` / `write_node_compressed` 的 `buf.push_str()` 热路径保持不变。

#### Scenario: serialize_expanded 使用 fold

- **WHEN** `serialize_expanded` 序列化节点列表为 CSS 字符串
- **THEN** 外层通过 `fold` 累积字符串，不使用 `let mut result = String::new(); for ... { result.push('\n'); write_node_expanded(&mut result, ...) }`

#### Scenario: write_node_expanded 保持 buf.push_str

- **WHEN** `write_node_expanded` 写入单个节点到缓冲区
- **THEN** 继续使用 `buf.push_str()` 直接写入，不改为 `format!` + `collect` + `join`（性能优化保留）

### Requirement: merge_at_rules 使用 fold 替代 for 循环

`merge_at_rules` 的 `for node in nodes { ... result.push(...) }` 循环 SHALL 改为 `nodes.iter().fold(Vec::new(), |mut result, node| { ...; result })`。

#### Scenario: merge_at_rules 使用 fold

- **WHEN** `merge_at_rules` 合并嵌套 @media 等规则
- **THEN** 通过 `fold` 累积结果，不使用 `let mut result = Vec::new(); for ... { result.push(...) }`

### Requirement: Map 内建函数使用 fold 替代 for 循环

`nested_map_merge`、`nested_map_set`、`deep_merge_maps`、`map_deep_remove` SHALL 使用 `iter().fold(Vec::new(), |mut acc, (k, v)| { ... })` 替代 `let mut result = Vec::new(); for (k, v) in map { ... result.push(...) }`。`map-get` SHALL 使用 `args[1..].iter().try_fold(args[0].clone(), |current, key| { ... })` 替代 `let mut current = args[0].clone(); for key in &args[1..] { ... }`。

#### Scenario: nested_map_merge 使用 fold

- **WHEN** `nested_map_merge` 合并嵌套 Map
- **THEN** 通过 `fold` 累积结果，不使用 `let mut result = Vec::new(); let mut found = false; for (k, v) in map { ... }`

#### Scenario: map-get 使用 try_fold

- **WHEN** `map-get` 遍历多级 key 路径
- **THEN** 通过 `try_fold` 传递当前值，不使用 `let mut current = args[0].clone(); for key in &args[1..] { ... }`
