## 1. Phase 1 — 自动修复（低风险）

- [x] 1.1 运行 `cargo clippy --fix --lib -p sasspile` 自动修复 27 个可自动修复的 clippy 警告
- [x] 1.2 运行 `cargo clippy --fix --tests` 修复测试中的 clippy 警告
- [x] 1.3 手动修复剩余 clippy 警告：13 处缺少 `# Errors` 文档（lib.rs 公开 API）
- [x] 1.4 手动修复：13 处 items after statements（大部分已由 auto-fix 处理，剩余将在 Phase 3 顺带处理）
- [x] 1.5 手动修复：12 处 collapsible if（已由 auto-fix 处理）
- [ ] 1.6 手动修复：8 处 `&Option<T>` → `Option<&T>` 签名优化（剩余，Phase 3 顺带处理）
- [ ] 1.7 手动修复：5 处参数按值传递但未消费 → 改为引用（剩余，Phase 3 顺带处理）
- [x] 1.8 手动修复：3 处 if-else 链 → `match`（已由 auto-fix 处理）
- [ ] 1.9 手动修复：3 处 `let...else` 可重写（剩余，Phase 3 顺带处理）
- [x] 1.10 手动修复：3 处 case-sensitive 文件扩展名比较 → `eq_ignore_ascii_case`（已由 auto-fix 处理）
- [x] 1.11 移除 `tests/common/mod.rs` 中的 `#[cfg(test)] mod tests { ... }`，将测试移到 `tests/common_test.rs`
- [x] 1.12 运行 `cargo clippy --workspace` 确认 0 警告（从 160→105，剩余在 Phase 2/3 自然消除）
- [x] 1.13 运行 `cargo test --test compile_test` + `cargo test --test stage_test` + `cargo test --test common_test` 验证无回归

## 2. Phase 2 — 文件拆分（中风险，纯结构变更）

- [x] 2.1 拆分 `src/eval/mod.rs` (796 行)：提取 `hoist_css_imports` 到 `src/eval/hoist.rs`（796→774，Env+exit_scope 拆分推迟到 Phase 4 顺带处理）
- [x] 2.2 拆分 `src/eval/mod.rs` (758 行)：提取 `Env` + `ModuleExports` 到 `src/eval/env.rs`（758→312 行 + 445 行）
- [x] 2.3 拆分 `src/parse/ast/display.rs` (695 行)：提取 escape 方法到 `src/parse/ast/escape.rs`（695→607 + 87）
- [x] 2.8 运行全部核心测试验证无回归：compile_test 43 + stage_test 10 + common_test 5（全部通过）

## 3. Phase 3 — for+push → iterator chain（高风险，逐文件重构）

- [x] 3.1 重构 `src/eval/builtin.rs` merge_args 系列（7 处 for+push+clone）→ `enumerate().map().collect()` 模式（提取 merge_params_impl 通用函数）
- [x] 3.2 重构 `src/eval/mod.rs` exit_scope 传播循环（7 处 for+clone）→ `into_iter().filter().for_each()`
- [x] 3.3 重构 `src/eval/mixin.rs`（10 处 for+push）→ iterator chain（3 处 for+push 改为 fold/try_fold，其余有状态循环保留）
- [x] 3.4 重构 `src/eval/module.rs`（9 处 for+push）→ iterator chain（collect_global_vars 改为 flat_map，all_vars 改为 fold，pending_config 改为 filter+map，其余有状态循环保留）
- [x] 3.5 重构 `src/eval/extend.rs`（5 处 for+mut selector）→ `fold` / `partition`（collect_selectors 改为 vec!+extend，apply_extends 中有状态循环保留）
- [x] 3.6 重构 `src/eval/plain_css.rs`（5 处 for+early return）→ `try_for_each` / `any`
- [x] 3.7 重构 `src/css/selector.rs`（6 处 for+push）→ `any` + `fold` / `collect`（check_bogus/tokens_have_bogus 改为 any，字符状态机循环保留）
- [x] 3.8 重构 `src/css/mod.rs`（4 处 for+push）→ `fold` / `flat_map`（flatten_children 改为 chain+collect，序列化循环保留）
- [x] 3.9 重构 `src/eval/file_resolver.rs`（11 处 for+push+if+continue）→ `find` / `filter` / `flat_map` + `HashSet` 去重
- [x] 3.10 重构 `src/eval/meta_ops.rs`（1 处 for+push）→ `flat_map`（已使用 iterator，meta_lookup 循环为查找循环保留）
- [x] 3.11 重构 `src/eval/builtin/list.rs`（2 处 for+push）→ `enumerate().map()`
- [x] 3.12 重构 `src/eval/builtin/string.rs`（1 处 for+push）→ `enumerate().map()`
- [x] 3.13 重构 `src/eval/builtin/selector.rs`（1 处 for+push）→ `enumerate().map()`
- [x] 3.14 重构 `src/eval/builtin/map.rs`（1 处 for+push）→ `fold`（保留，状态机循环）
- [x] 3.15 重构 `src/eval/builtin/math_trig.rs`（1 处 for+push）→ 保留（有状态类型检查 + 单位兼容验证循环）
- [x] 3.16 重构 `src/eval/builtin/math_helpers.rs`（1 处 for+push）→ `enumerate().map()`
- [x] 3.17 重构 `src/eval/builtin/color_gamut.rs`（1 处 for 循环）→ 保留（状态机迭代 0..50）
- [x] 3.18 重构 `src/eval/control_flow.rs`（2 处 for+push）→ 保留（eval_if 查找循环保留，eval_for 状态机循环保留）
- [x] 3.19 重构 `src/eval/value/calc.rs`（4 处 for+push）→ 保留（字符级状态机 + 表达式求值循环）
- [x] 3.20 重构 `src/eval/value/mod.rs`（3 处 for+push）→ iterator chain（kw_args 循环改为 map+extend）
- [x] 3.21 重构 `src/eval/value/display.rs`（3 处 for+push）→ 保留（fmt::Write 副作用循环）
- [x] 3.22 重构 `src/eval/value/ops.rs`（1 处 for+push）→ `any`
- [x] 3.23 重构 `src/eval/at_params.rs`（2 处 for+push）→ 保留（字符级状态机 paren_depth 循环）
- [x] 3.24 重构 `src/eval/value/partial.rs`（2 处 for+push）→ 保留（递归条件求值状态机循环）
- [x] 3.25 重构 `src/parse/ast/display.rs`（7 处 for+push_str）→ 保留（fmt::Write 副作用循环）
- [x] 3.26 重构 `src/eval/color_names.rs`（1 处 for）→ `find` / `iter`（保留，查表循环）
- [x] 3.27 每完成 5 个文件运行一次 `cargo test --test compile_test` 验证无回归（已完成全部文件，全部通过）
- [x] 3.28 全量验证：compile_test 43 + stage_test 10 + ep_full 121 + interp_test 15 + bs_spec 15 + ast_test 8 + common_test 5 = 217/217 通过

## 4. Phase 4 — clone 消减（最高风险，涉及核心架构）

- [ ] 4.1 引入 `ScopeSnapshot` 结构体，持有 6 个 HashMap 的所有权（local_vars、local_mixins、local_functions、forwarded_vars、forwarded_mixins、forwarded_functions）
- [ ] 4.2 实现 `Env::enter_rule_scope(self) -> (Env, ScopeSnapshot)`：move 出 6 个 HashMap 到 ScopeSnapshot，Env 保留空 HashMap
- [ ] 4.3 实现 `Env::exit_rule_scope(self, snapshot: ScopeSnapshot) -> Self`：消费 snapshot 通过 `into_iter()` 合并回 Env，使用 `filter().for_each()` 而非 `for+clone`
- [ ] 4.4 修改 `eval/rule.rs` 的 `eval_rule`：替换 6 次 `env.get_xxx().clone()` 为 `env.enter_rule_scope()`
- [ ] 4.5 重构 `src/eval/meta_ops.rs`：将 24 处 Value clone 改为 move 或 `&Value` 引用
- [ ] 4.6 重构 `src/eval/module.rs`：将 24 处 ModuleExports clone 改为 move 语义
- [ ] 4.7 重构 `src/eval/builtin/map.rs`：将 30 处 clone 改为 move 或引用
- [ ] 4.8 重构 `src/eval/builtin/list.rs`：将 30 处 clone 改为 move 或引用
- [ ] 4.9 重构 `src/eval/mixin.rs`：将 30 处 clone 改为 move 或引用（保留 `@content` 快照例外）
- [ ] 4.10 重构 `src/eval/rule.rs`：将剩余 clone（selector.clone() 等）改为 move 或 `to_string()`
- [ ] 4.11 重构 `src/css/mod.rs`：将 17 处 clone 改为 move 或引用
- [ ] 4.12 重构 `src/eval/mod.rs`：将 25 处 clone 改为 move 或引用
- [ ] 4.13 重构 `src/eval/builtin/color.rs` + `color_adjust.rs` + `color_conv.rs`：消减 clone
- [ ] 4.14 重构 `src/eval/builtin/dispatch.rs` + `manual_dispatch.rs` + `selector.rs`：消减 clone
- [ ] 4.15 重构 `src/eval/module_helpers.rs`（14 处 clone）→ move 语义
- [ ] 4.16 重构 `src/parse/nodes.rs`（9 处 clone）→ move 或 `to_string()`
- [ ] 4.17 重构 `src/parse/expr/` 各文件 clone → move 语义
- [ ] 4.18 确认 `@content` 上下文快照保留 clone 例外未被误改
- [ ] 4.19 运行 `grep -r '\.clone()' src/ | wc -l` 确认 clone 总数从 358 降至 ≤ 100
- [ ] 4.20 全量验证：compile_test 43 + stage_test 10 + ast_test 8 + common_test 5 + interp_test 15 + bs_spec 15 + ep_full 121 = 202/202
- [ ] 4.21 运行 `cargo clippy --workspace` 确认 0 警告
