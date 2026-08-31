## Context

sasspile 管线层已实现类型状态机链式调用（`Source → Lexed → Parsed → Evaluated → Serialized`），每个阶段 `self → NextStage`。但阶段内部仍广泛使用命令式循环累积器：

```rust
// 当前模式——eval_nodes
let mut css = Vec::new();
let mut env = env;
for node in nodes {
    let (out, new_env) = Self::eval_node(node, env)?;
    css.append(&mut out);
    env = new_env;
}
```

eval 模块统计：254 处 `mut`、288 处 `clone`、20 处 `&mut`。其中循环累积器类约占 100 处 `mut`，可通过 `try_fold` / `fold` 消除。

## Goals / Non-Goals

**Goals:**
- 将 `eval_nodes` / `eval_for` / `eval_each` / `hoist_css_imports` 等循环累积器改为 `try_fold` / `fold` / `partition`
- 将 `Evaluated::serialize` 从 `&self` 改为 `self`（消费所有权）
- 将 `flatten_nodes` 改为 `flat_map` + `collect`
- 消除约 100 处 `mut` 累积器
- 保持 202/202 测试全通过 + sass-spec 2828/5362 基线不变

**Non-Goals:**
- 不改 Lexer/Parser 的 `&mut self`（递归下降解析器的标准实践，性能最优）
- 不改 `write_node_expanded` / `write_node_compressed` 的 `&mut String`（最高效的字符串拼接方式）
- 不改 `eval_while` 的 `loop`（无界循环不适合 fold，保留 `loop` 但消除内部 `mut css`）
- 不消除 `Rc<ModuleExports>`（共享语义合理，消除代价过大）
- 不改变任何 spec 级行为（纯内部重构）

## Decisions

### D1: `eval_nodes` 用 `try_fold` 替代 for 循环

**选择**：`try_fold((Vec::new(), env), |(mut css, env), node| { ... })`

**理由**：
- `eval_node` 返回 `Result`，`try_fold` 是标准选择
- `(Vec<CssNode>, Env)` 作为 fold 状态，天然表达 "数据流过管线"
- 消除 `let mut env = env` 和 `let mut css`

**替代方案**：用 `for` 循环 + `?`（当前方案）——保留 mut，但可读性好。否决：与链式风格不一致。

### D2: `eval_for` 用 range + `try_fold`

**选择**：`(start..stop).step_by(step as usize).try_fold(...)`

**理由**：
- `@for` 循环是有界 range，天然适合 fold
- step 方向通过 `step_by` 处理（正负 step 转换为 usize）

**注意**：i64 负数 step 需要先计算方向，然后用正数 `step_by`。如果 `start > end`，step = -1，转换为 `(end+1..=start).rev()`。

### D3: `eval_each` 用 `try_fold`

**选择**：`items.iter().try_fold((Vec::new(), env), |(mut css, env), item| { ... })`

**理由**：直接将 for 循环体搬入 fold 闭包。

### D4: `eval_while` 保留 `loop` 但消除 `mut css`

**选择**：保留 `loop { ... }` 结构，但用 `std::mem::take` + extend 替代 `mut css`

**理由**：无界循环不能用 `try_fold`。强行用 `std::iter::from_fn` + `take_while` 会更丑且编译器难优化。保留 `loop` 是务实选择。

### D5: `hoist_css_imports` 用 `partition`

**选择**：`nodes.into_iter().partition(|n| is_css_import(n))`

**理由**：当前 for 循环本质上就是 partition——先递归处理嵌套，再按 `is_css_import` 分流。用 `partition` + 先 `map` 递归处理最清晰。

**注意**：递归处理需要在 partition 前用 `map` 完成。即 `nodes.into_iter().map(|n| match n { nested => flatten_recursive(n), other => other }).collect::<Vec<_>>().into_iter().partition(...)`。

### D6: `eval_rule` 封装 `RuleBuilder` 状态结构

**选择**：创建 `struct RuleBuilder { result: Vec<CssNode>, current_decls: Vec<CssNode>, root_nodes: Vec<CssNode> }`，实现 `push(node)` 方法，然后用 `fold`

**理由**：3 个累积器直接放进 fold 元组会让签名变成 `(Vec, Vec, Vec)` 极难读。封装成 struct + push 方法后，fold 闭包只需 `|builder, node| { builder.push(node); builder }`。

### D7: `flatten_nodes` 用 `flat_map`

**选择**：`nodes.iter().flat_map(|n| Self::flatten_node(n)).collect()`

**理由**：`flatten` 本质是 flat_map——每个节点展开为零或多个扁平节点。当前递归 for 循环天然对应 flat_map。

### D8: `Evaluated::serialize` 改为 `self`

**选择**：`pub fn serialize(self, style: OutputStyle) -> Serialized`

**理由**：与管线链式一致——`evaluate()` 返回 `Evaluated`（owned），`serialize` 应消费它。当前 `&self` 迫使调用方保留引用。改为 `self` 后管线完全 owned 链式。

**影响**：`lib.rs` 的 `compile` 函数无需改动（已经是 `evaluated.serialize(style)` 链式调用），但 `Evaluated` 不再 `Clone`（或保留 Clone 但 serialize 消费）。

## Risks / Trade-offs

- **[fold 闭包内 `mut css`]** → fold 闭包仍然需要 `mut css` 来 `extend`。这是 Rust 的限制——`try_fold` 的闭包参数是 by-value，但闭包内可以 `mut`。这不是"真正的 mut"——它是 fold 的局部状态，没有跨迭代持久化。
- **[性能]** → `try_fold` 和 for 循环在 release 模式下性能等价（编译器优化后都是同一个循环）。`partition` 和 `flat_map` + `collect` 可能多一次遍历，但差异在微秒级。
- **[eval_for 的 range 转换]** → i64 → range + step_by 需要处理负数 step。如果 start > end，需要反转 range。这增加了一点复杂度，但逻辑清晰。
- **[RuleBuilder 引入新类型]** → 新增一个 `struct RuleBuilder` 增加了代码量。但它封装了 3 个累积器的交互逻辑，比裸 fold 元组更可维护。
- **[回归风险]** → 纯机械重构，不改逻辑。但 `eval_rule` 的状态转移较多，需要仔细验证。每个 task 完成后立即跑 `compile_test` + `stage_test` 确认无回归。
