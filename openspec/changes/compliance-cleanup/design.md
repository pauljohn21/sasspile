## Context

sasspile 项目经过多轮 AI 迭代后积累了大量违反函数式 Rust 规范的代码。当前状态：

- **6 个源文件超过 500 行上限**（最大 796 行），违反单文件 ≤ 500 行规则
- **~100 处 `for+push` 命令式累积**分布在 32 个文件中，违反"集合变换用 map/filter/collect"规则
- **358 处 `clone()`**，其中 `eval/rule.rs` 的 `eval_rule` 单函数 6 次 HashMap 深拷贝用于 scope 保存/恢复
- **160 个 clippy pedantic 警告**未修复
- **`tests/common/mod.rs` 存在 `#[cfg(test)]` 内联测试**

核心根因是 `Env::exit_scope` 的"保存→执行→合并"模式：`eval_rule` 需要 clone 6 个 HashMap 来快照 scope，因为 `eval_nodes` 消费 env（move 语义），但 scope 需要在规则体执行后恢复。

## Goals / Non-Goals

**Goals:**
- 将所有源文件缩减到 ≤ 500 行（组件推荐 ≤ 300 行）
- 将所有 `for+push` 模式重构为 iterator chain（`map`/`filter`/`fold`/`try_fold`/`partition`）
- 消减 `clone()` 使用，特别是 `exit_scope` 的 6 次 HashMap clone
- 消除所有 clippy pedantic 警告
- 移除 `tests/common/mod.rs` 的 `#[cfg(test)]`

**Non-Goals:**
- Parser 的 `&mut self` 不在本次重构范围内（有状态状态机是 Rust 惯用法，保留）
- 不重构 `Env` 的整体数据结构（仍使用 `HashMap`，不改为 `im::HashMap` 或其他持久化数据结构）
- 不修改公开 API（`compile`、`compile_expanded` 等签名不变）
- 不追求 `clone()` 归零（某些场景如 `@content` 上下文快照需要 clone）

## Decisions

### D1: Env scope 快照从 clone 改为 move + 结构体

**决策**：引入 `ScopeSnapshot` 结构体，`enter_rule_scope(self) -> (Env, ScopeSnapshot)` 消费 Env 并 move 出 scope 数据，`exit_rule_scope(self, snapshot: ScopeSnapshot) -> Env` 合并回。

**替代方案**：使用 `Rc<HashMap>` COW — 已在 `mut-env-refactor` 归档中弃用，因为模仿 GC 模式。

**理由**：move 语义零拷贝，符合 sasspile 函数式设计哲学。`ScopeSnapshot` 只持有 6 个 HashMap 的所有权（move），不是深拷贝。

### D2: 文件拆分策略 — 按功能边界，不按行数机械切分

**决策**：
- `eval/mod.rs` (796 行) → 拆出 `eval/scope.rs`（Env + exit_scope）、`eval/hoist.rs`（hoist_css_imports）
- `parse/ast/display.rs` (695 行) → 按类型拆分到 `display_value.rs`、`display_node.rs`
- `eval/builtin/color_adjust.rs` (663 行) → 拆出 `color_adjust_ops.rs`（adjust/change/scale 操作）
- `eval/module.rs` (657 行) → 拆出 `module_load.rs`（load_module）、`module_bind.rs`（bind_exports）
- `eval/builtin/color.rs` (572 行) → 拆出 `color_create.rs`（rgb/hsl/hwb 创建）
- `css/mod.rs` (514 行) → 拆出 `flatten.rs`（flatten_nodes）、`serialize_expanded.rs`

**理由**：按功能边界拆分使每个文件内聚，便于后续维护。

### D3: for+push → iterator chain 的分类处理

**决策**：按模式分类处理：

| 模式 | 替代方案 | 典型文件 |
|------|---------|---------|
| `for+push(单一变换)` | `.map(f).collect()` | builtin.rs merge_args |
| `for+push+filter` | `.filter(pred).map(f).collect()` | meta_ops.rs |
| `for+insert(clone)` | `.into_iter().filter().for_each(insert)` | mod.rs exit_scope |
| `for+early return(错误传播)` | `.try_fold(acc, \|acc, x\| ...)` | plain_css.rs |
| `for+partition` | `.partition(pred)` | extend.rs |
| `for+mut String` | `.fold(String::new(), \|acc, x\| ...)` | selector.rs |

**例外**：Parser 状态机循环（`while let Some(t) = self.peek()`）保留，不在此范围内。

### D4: Clippy 修复策略 — 自动修复优先

**决策**：
1. 先运行 `cargo clippy --fix --lib -p sasspile` 自动修复 27 个可自动修复的警告
2. 手动修复剩余 133 个警告（补充 `# Errors` 文档、拆分过长函数、改 `&Option<T>` → `Option<&T>` 等）
3. 不使用 `#[allow(clippy::xxx)]` 抑制警告（除非有明确正当理由）

### D5: clone 消减优先级

**决策**：按 clone 次数和影响范围排序：
1. `eval/rule.rs` 的 6 次 scope clone → D1 的 `ScopeSnapshot` move 方案
2. `eval/builtin.rs` merge_args 的 `pos_args[i].clone()` → 改用 `get(i).cloned()` 或 move
3. `eval/meta_ops.rs` 的 24 处 Value clone → 分析所有权，改为 move 或 `&Value`
4. `eval/module.rs` 的 24 处 ModuleExports clone → 改为 move（`ModuleExports` 已实现 `Clone`，但大部分场景可 move）
5. 其余低频 clone 逐文件处理

## Risks / Trade-offs

- [重构涉及核心求值路径] → 每个 Phase 完成后运行 compile_test + stage_test + ep_full 验证，确保 202/202 通过
- [文件拆分可能引入循环依赖] → 使用 `pub(crate)` 可见性，模块间通过 `super::*` 引用
- [exit_scope move 重构可能影响 @content 上下文快照] → 保留 `@content` 场景的 clone（规则已允许此例外）
- [for+push → iterator 可能改变执行顺序] → 使用 `try_fold` 保持短路语义，`fold` 保持顺序
- [clippy 自动修复可能引入语义变更] → 每次自动修复后运行测试验证

## Migration Plan

分 4 个 Phase 逐步推进，每个 Phase 独立验证：

1. **Phase 1 — 自动修复**：`cargo clippy --fix` + 移除 `#[cfg(test)]`（低风险，立即可做）
2. **Phase 2 — 文件拆分**：拆分 6 个超 500 行文件（中风险，纯结构变更不改逻辑）
3. **Phase 3 — for+push → iterator**：逐文件重构 for 循环（高风险，需逐文件测试）
4. **Phase 4 — clone 消减**：Env scope 快照 move 重构 + Value/ModuleExports clone 消减（最高风险，涉及核心架构）

**回滚策略**：每个 Phase 独立 git commit，如测试失败可 `git revert` 单个 Phase。

## Open Questions

- `exit_scope` 重构后，`@content` 上下文快照是否需要保留 clone？（当前规则允许，预计保留）
- `parse/ast/display.rs` 的 695 行中有大量 `match` 分支，拆分后是否影响可读性？（预计按类型拆分后更清晰）
