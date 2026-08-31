## Context

chain-reaction 重构已将核心 eval 管线（eval_nodes/eval_for/eval_each/eval_rule/hoist_css_imports/flatten_nodes）改造为 try_fold/fold/flat_map 链式风格。但项目其余模块仍保留大量命令式循环+push 模式，涉及约 25 个函数、~450 行代码。

当前状态：
- **CSS 序列化器**（`css/mod.rs`）：`flatten_children`、`serialize_expanded`、`serialize_compressed`、`merge_at_rules` 使用 `for + push/extend`
- **选择器规范化**（`css/selector.rs`）：4 个函数使用 `while i < chars.len() + push` 手动字符迭代
- **Map 内建函数**（`eval/builtin/map.rs`）：5 个函数使用 `for + push` 和 `let mut found = false` 模式
- **color_adjust.rs**：15 个 adjust/change/scale 函数有 `let mut x = ...; if let Some(v) = ... { x = ... }` 重复模式
- **杂项**：`nest_rule_in_children`、`eval_import` CSS 分支、`to_scss` If 分支、`selector-simple-selectors`

## Goals / Non-Goals

**Goals:**
- 将所有 `let mut result = Vec::new(); for ... { result.push(...) }` 模式改为 `fold` / `collect` / `flat_map` 链式风格
- 将 color_adjust.rs 的 mut 变量赋值链提取为 `apply_kw` helper，消除重复
- 保持所有测试通过：202/202 + sass-spec 2902/5362

**Non-Goals:**
- 不重构解析器（`parse/`）的 `while let Some(t) = self.peek()` 循环 — 递归下降解析器本质是有状态状态机
- 不重构词法分析器（`lex/mod.rs`）的 `loop` — 状态机特征
- 不重构 CSS 序列化器内部的 `buf.push_str()` 热路径 — 已注释说明有意为之的性能优化
- 不改变任何函数的行为语义 — 纯风格重构

## Decisions

### D1: flatten_children 改为 flat_map + chain

**决策**：`flatten_children` 的 `for + push/extend` 改为 `children.iter().flat_map(|child| { ... }).collect()`，内部用 `chain` 拼接选项。

**理由**：`flat_map` 天然处理 "一个输入产生多个输出" 的场景，比 `for + extend` 更声明式。

### D2: serialize_expanded/compressed 外层 for 改为 fold

**决策**：外层 `for (i, n) in nodes.iter().enumerate()` 改为 `nodes.iter().enumerate().fold(String::new(), |acc, (i, n)| { ... })`。内部的 `write_node_expanded`/`write_node_compressed` 保持 `buf.push_str()` 热路径不变。

**理由**：外层 fold 消除 `let mut result = String::new()` 可变绑定；内部 `push_str` 是性能优化保持不变。替代方案 `iter().map(...).collect::<Vec<String>>().join("\n")` 会导致多次分配，性能更差。

### D3: Map 内建函数改为 fold

**决策**：`nested_map_merge`/`nested_map_set`/`deep_merge_maps`/`map_deep_remove` 的 `for + push` 改为 `iter().fold(Vec::new(), |mut acc, (k, v)| { ... })`。`map-get` 的 `for key in &args[1..]` 改为 `args[1..].iter().try_fold(args[0].clone(), |current, key| { ... })`。

**理由**：fold 保持语义等价，消除 `let mut found = false` 标志（fold 可以通过 acc 的状态隐式判断）。替代方案用 `scan` 不够自然。

### D4: color_adjust.rs 提取 apply_kw helper

**决策**：提取一个 helper 函数：
```rust
fn apply_kw(initial: f64, kw: &HashMap<String, Value>, key: &str, f: impl Fn(f64, f64) -> f64) -> Result<f64>
```
将 15 个 adjust/change/scale 函数中的 `let mut x = ...; if let Some(v) = get(kw, key)? { x = f(x, v); }` 模式统一替换。

**理由**：消除约 60 行重复代码。替代方案用 builder pattern（`ColorBuilder::new(c).adjust("lightness", v).adjust("chroma", v)`）更优雅但改动范围太大。

### D5: 选择器规范化保持 while 循环

**决策**：`tokenize_selector_with_pseudo` 等 4 个字符级处理函数保持 `while i < chars.len()` 循环结构，但内部消除 `let mut result = Vec::new()` / `let mut result = String::new()`，改用 `chars.iter().fold(...)` 或 `chars.iter().scan(...).collect()`。

**理由**：这些函数有复杂的嵌套深度跟踪（`depth += 1`）和回溯逻辑，完全消除 `while` 会让代码更难读。只消除外层可变绑定即可。

## Risks / Trade-offs

- **[Risk] fold 闭包中 mut acc** → fold 的闭包参数 `|mut acc, item|` 仍然需要 `mut`，但这是 fold 的惯用法，比 `let mut result = Vec::new()` 更好 — `mut` 的作用域限定在闭包内
- **[Risk] color_adjust helper 泛型可能影响性能** → `impl Fn(f64, f64) -> f64` 是零成本抽象，编译器内联后无额外开销
- **[Risk] 选择器规范化的 while 循环部分保留** → 这是 trade-off：完全函数式化会损害可读性，保留 while + 消除外层 mut 是合理折中
- **[Risk] 重构引入 subtle bug** → 每个批次后运行完整测试套件（compile_test + ep_full）确保零回归
