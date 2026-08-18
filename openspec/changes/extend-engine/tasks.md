# Extend 引擎实现任务

## Phase 1: ExtensionStore 基础 + eval_use_rule 传递（预估 2h）

- [ ] 1.1 创建 `src/selector/extend/` 目录结构：`mod.rs`、`unify.rs`、`transitive.rs`、`merge.rs`
- [ ] 1.2 在 `extend/mod.rs` 中定义 `ExtensionStore`、`Extension`、`ModuleId` 类型
- [ ] 1.3 实现 `ExtensionStore::add()` — 按 extendee 选择器字符串索引 extensions
- [ ] 1.4 实现 `ExtensionStore::next_module_id()` / `push_module()` / `pop_module()` / `current_module_id()`
- [ ] 1.5 修改 `eval/mod.rs`：`evaluate` 创建 `ExtensionStore`，传递给 `eval_stmts`
- [ ] 1.6 修改 `eval/mod.rs`：`eval_use_rule` 将 `&mut store` 传递给子模块 `eval_stmts`（替代 `&mut Vec::new()`）
- [ ] 1.7 修改 `eval/mod.rs`：`eval_stmt` 的 `ExtendRule` 分支改为调用 `store.add()`
- [ ] 1.8 修改 `eval/func.rs`：`call_user_function` 传递真实 extends（替代 `dummy_extends`）
- [ ] 1.9 删除 `serialize.rs` 中的 `apply_extends` / `apply_extends_to_rule` 函数
- [ ] 1.10 修改 `serialize.rs`：`serialize_with_style` 直接输出 `CssTree.rules`（extends 已应用）
- [ ] 1.11 实现 `extend/mod.rs`：`apply_to_rules()` — 遍历 CSS 规则，对 Style 规则的选择器应用 extends（调用旧的 `apply_extends_to_list` 逻辑）
- [ ] 1.12 修改 `eval/mod.rs`：`evaluate` 在 eval_stmts 后调用 `store.apply_to_rules()`
- [ ] 1.13 创建 `tests/extend_basic.rs`：测试 `upstream/near`、`upstream/placeholder`、`directives/extend/after_target` 场景

## Phase 2: 传递性解析 + 循环检测（预估 2h）

- [ ] 2.1 在 `extend/transitive.rs` 中实现 `resolve_transitive()` — BFS 遍历 extend 依赖图
- [ ] 2.2 实现 `visited: HashSet<String>` 循环检测
- [ ] 2.3 修改 `apply_to_rules`：对每条 extend 的结果递归应用其他 extends（传递性）
- [ ] 2.4 实现 `extend/transitive.rs` 中的 `build_extend_graph()` — 从 `ExtensionStore` 构建 extendee → extender 的邻接表
- [ ] 2.5 添加 tracing span：`extend_transitive` span，记录 start selector 和 visited count
- [ ] 2.6 创建 `tests/extend_transitive.rs`：测试 `extended/from_same_file`、`extended/from_other_file`、`diamond/dependency`、`180_basic_extend_loop`

## Phase 3: Compound/Complex 选择器统一（预估 4h）

- [ ] 3.1 在 `extend/unify.rs` 中实现 `unify_complex()` — complex 选择器统一入口
- [ ] 3.2 实现 `unify_compound_partial()` — compound 内 partial 替换（保留非匹配 simple selector）
- [ ] 3.3 实现 `weave_complex()` — 后代选择器交织（extender 前缀 + 替换部分 + 原选择器后缀）
- [ ] 3.4 实现组合器保留逻辑：`>` `+` `~` 在 weave 中正确传递
- [ ] 3.5 修改 `apply_extends_to_list`：调用 `unify_complex()` 替代旧的 `merge_selectors()`
- [ ] 3.6 添加 tracing span：`extend_unify` span，记录 extendee、extender、unified 结果
- [ ] 3.7 创建 `tests/extend_unify.rs`：测试 `nested-compound-unification`、`046`-`077` unification 系列

## Phase 4: 模块作用域隔离（预估 2h）

- [ ] 4.1 在 `ExtensionStore` 中实现 `module_graph: HashMap<ModuleId, HashSet<ModuleId>>`
- [ ] 4.2 修改 `eval_use_rule`：在进入子模块前 `store.push_module(id)`，退出时 `store.pop_module()`
- [ ] 4.3 在 `module_graph` 中记录当前模块 → 子模块的边
- [ ] 4.4 实现 `is_reachable(ext_module_id, rule_module_id)` — BFS 检查模块依赖可达性
- [ ] 4.5 修改 `apply_to_rules`：对每条 extend 检查 `is_reachable`，不可达则跳过
- [ ] 4.6 实现 placeholder 可见性检查：`%-` 前缀只对定义模块内可见
- [ ] 4.7 添加 tracing span：`extend_scope` span，记录 ext_module_id 和 rule_module_id
- [ ] 4.8 创建 `tests/extend_scope.rs`：测试 `scope/sibling`、`scope/diamond`、`scope/downstream`、`scope/private`

## Phase 5: 高级特性（预估 4h）

- [ ] 5.1 在 `extend/unify.rs` 中实现 `:is()` / `:matches()` / `:where()` 伪类穿透
- [ ] 5.2 递归将 extend 应用到伪类参数内部的选择器
- [ ] 5.3 在 `extend/merge.rs` 中实现 `remove_redundant()` — superselector 检测和移除
- [ ] 5.4 改进 `is_superselector` 实现为结构化匹配（替代字符串 startsWith）
- [ ] 5.5 实现 `remove_placeholders()` — 从最终选择器列表中移除 `%` 前缀选择器
- [ ] 5.6 实现 `deduplicate_simples()` — 合并 compound 内重复 simple（`.a.a` → `.a`）
- [ ] 5.7 实现 `@media` 内 extend 的独立作用域（同一 @media 内 extends 互不影响其他 @media）
- [ ] 5.8 添加 tracing span：`extend_dedup` span，记录 before/after count
- [ ] 5.9 创建 `tests/extend_advanced.rs`：测试 `pseudo.hrx`、`091_redundant_selector_elimination`、`187_basic_placeholder`、`extend-loop.hrx`

## Phase 6: 集成测试 + 回归验证（预估 1h）

- [ ] 6.1 运行 `sass-spec` 中 `directives/extend/` 全部测试用例
- [ ] 6.2 运行 `sass-spec` 中 `directives/use/extend/` 全部测试用例
- [ ] 6.3 运行 `sass-spec` 中 `non_conformant/extend-tests/` 核心测试用例（前 50 个）
- [ ] 6.4 验证 Element Plus / Bootstrap 编译无回归
- [ ] 6.5 验证所有现有 `cargo test` 通过
