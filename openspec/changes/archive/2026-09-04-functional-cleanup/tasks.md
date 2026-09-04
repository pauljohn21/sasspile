# Implementation Tasks

## Phase 1: scanner.rs — else-if 链消除（6 处）

- [x] 1.1 `scan_ident`：7 个 else-if 关键字分派 → `match text.to_lowercase().as_str()`
- [x] 1.2 `scan_string`：转义处理中的 else-if → match
- [x] 1.3 `scan_escape_ident`：已有 match，验证一致性
- [x] 1.4 `scan_at`：内嵌 else-if → match
- [x] 1.5 `scan_hash`：内嵌 else-if → match
- [x] 1.6 `lex/mod.rs`：1 处 else-if → match

## Phase 2: color_adjust.rs — legacy 函数重写（3 个函数）

- [x] 2.1 `adjust_legacy`：9 个 if-let + mut → `apply_kw`/`apply_pct_kw` 链式
- [x] 2.2 `change_legacy`：同上模式重写
- [x] 2.3 `scale_legacy`：同上模式重写
- [x] 2.4 提取 `has_hsl`/`has_hwb` 标志为 Option 链或 bool::then

## Phase 3: selector.rs — else-if 链消除（8 处）

- [x] 3.1 `remove_placeholder_not`：if-else 链 → match
- [x] 3.2 `has_bogus_combinators` 系列：if-else → match
- [x] 3.3 `tokenize_selector_with_pseudo`：if-else → match
- [x] 3.4 `tokens_have_bogus`：if-else → match

## Phase 4: color.rs — else-if 链消除（8 处）

- [x] 4.1 颜色空间分派 if-else → match ColorSpace
- [x] 4.2 颜色转换函数 if-else → match

## Phase 5: color_conv_ops.rs — else-if 消除（6 处）

- [x] 5.1 颜色空间转换分派 → match
- [x] 5.2 通道提取 if-else → match

## Phase 6: mixin.rs — else-if + for+push 消除（5+3 处）

- [x] 6.1 mixin 参数分派 else-if → match
- [x] 6.2 for+push → try_fold/collect

## Phase 7: value/mod.rs — for+push + if-let 消除

- [x] 7.1 `eval_args`：for + push + extend → try_fold
- [x] 7.2 `eval_value`：if-let 链 → match
- [x] 7.3 `dispatch_function`：else-if → match

## Phase 8: parse/ 模块 — else-if 消除

- [x] 8.1 `parse/at_rules.rs`（4 处）：@规则分派 → match
- [x] 8.2 `parse/ast_impl.rs`（4 处）：AST 节点分派 → match
- [x] 8.3 `parse/ast/color_fmt.rs`（4 处）：颜色格式 → match
- [x] 8.4 `parse/ast/display.rs`（3 处 + 6 个 for 循环）：Display → match + 迭代器链
- [x] 8.5 `parse/nodes.rs`（2 处）：→ match
- [x] 8.6 `parse/params.rs`：for+push → collect
- [x] 8.7 `parse/expr/prefix.rs`（1 处）：→ match

## Phase 9: value/display.rs + value/partial.rs — else-if 消除

- [x] 9.1 `value/display.rs`（4 处）：Value Display 分派 → match
- [x] 9.2 `value/partial.rs`（2 处）：→ match

## Phase 10: css/mod.rs — for+push + &mut 消除

- [x] 10.1 `write_node_expanded`：`&mut String` → 返回 String（filter_map + collect 替代 for+push）
- [x] 10.2 `write_node_compressed`：同上（for_each 替代 for 循环递归调用）
- [x] 10.3 `process_node`：`&mut ScanState` 保留（内部状态机，合理）
- [x] 10.4 for+push 序列化 → 迭代器链

## Phase 11: 剩余 1-2 处 else-if 文件批量处理

- [x] 11.1 `eval/builtin/string.rs`（2 处）
- [x] 11.2 `eval/builtin/math.rs`（2 处）
- [x] 11.3 `eval/builtin/color.rs`（1 处）
- [x] 11.4 `eval/builtin/map.rs`（1 处）
- [x] 11.5 `eval/builtin/list.rs`（1 处）
- [x] 11.6 `eval/value/calc.rs`（1 处）
- [x] 11.7 `eval/rule.rs`（1 处）
- [x] 11.8 `eval/plain_css.rs`（1 处）
- [x] 11.9 `eval/import.rs`（1 处）
- [x] 11.10 `eval/forward.rs`（1 处）
- [x] 11.11 `eval/control_flow.rs`（1 处）
- [x] 11.12 `eval/at_params.rs`（1 处）
- [x] 11.13 `parse/at_rules_modules.rs`（1 处）
- [x] 11.14 `parse/ast/mod.rs`（1 处）
- [x] 11.15 `css/selector_parser.rs`（1 处）
- [x] 11.16 `css/selector_ast.rs`（1 处）

## Phase 12: file_resolver.rs — for 循环消除（7 处）

- [x] 12.1 嵌套 for 路径搜索 → flat_map + flatten
- [x] 12.2 `try_resolve_dir` for 循环 → iterator chain
- [x] 12.3 `check_explicit_ext_conflicts` for+push → flat_map + flatten + collect
- [x] 12.4 `check_no_ext_conflicts` for+push → filter_map + chain + collect
- [x] 12.5 `check_resolve_ambiguity` for + return Err → try_for_each
- [x] 12.6 `normalize_path` for + push → fold

## Phase 13: 最终验证

- [x] 13.1 `cargo check` 零错误零警告
- [x] 13.2 全量核心测试：105/105（43+10+8+5+15+15+9）
- [x] 13.3 ep_full：121/121
- [x] 13.4 sass-spec 全量：3366/5624 ≥ 3366 ✓
- [x] 13.5 `grep -rn "else if" src/ | wc -l` = 4 ≤ 5（全部为注释/字符串字面量）
- [x] 13.6 `codegraph sync` 完成
- [x] 13.7 裸 `if` 审计：剩余 10 处全部为 match guard 或 Rust 2024 let chains（不可消除）
