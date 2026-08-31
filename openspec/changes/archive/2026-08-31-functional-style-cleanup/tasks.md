## 1. CSS 序列化器（css/mod.rs）

- [x] 1.1 `flatten_children` 改为 `children.iter().flat_map(|child| { ... }).collect()` 替代 `for + push/extend`
- [x] 1.2 `serialize_expanded` 外层 `for + push` 改为 `nodes.iter().enumerate().fold(String::new(), ...)`
- [x] 1.3 `serialize_compressed` 外层 `for + write_node_compressed` 改为 `nodes.iter().fold(String::new(), ...)`
- [x] 1.4 `merge_at_rules` 内部 `for + push` 改为 `nodes.iter().fold(Vec::new(), ...)`
- [x] 1.5 运行 `cargo test --test compile_test` 确认 43/43 通过

## 2. Map 内建函数（eval/builtin/map.rs）

- [x] 2.1 `nested_map_merge` 的 `for + push + found` 改为 `iter().fold`
- [x] 2.2 `nested_map_set` 的 `for + push + found` 改为 `iter().fold`
- [x] 2.3 `deep_merge_maps` 的 `for + push` 改为 `iter().fold`
- [x] 2.4 `map_deep_remove` 的 `for + push` 改为 `iter().fold`
- [x] 2.5 `map-get` 的 `for key in &args[1..]` 改为 `try_fold`
- [x] 2.6 运行 `cargo test --test compile_test -- map` 确认 Map 相关测试通过

## 3. color_adjust.rs helper 提取

- [x] 3.1 定义 `apply_kw` helper 函数：`fn apply_kw(initial: f64, kw: &HashMap<String, Value>, key: &str, f: impl Fn(f64, f64) -> f64) -> Result<f64>`
- [x] 3.2 `adjust_oklab` / `change_oklab` / `scale_oklab` 改用 `apply_kw`
- [x] 3.3 `adjust_lch` / `change_lch` / `scale_lch` 改用 `apply_kw`
- [x] 3.4 `adjust_lab` / `change_lab` / `scale_lab` 改用 `apply_kw`
- [x] 3.5 `adjust_modern_rgb_space` / `change_modern_rgb_space` / `scale_modern_rgb_space` 改用 `apply_kw`
- [x] 3.6 `adjust_legacy` / `change_legacy` / `scale_legacy` 改用 `apply_kw`（注意 has_hsl/has_hwb 标志的处理）
- [x] 3.7 `adjust_oklch` / `change_oklch` / `scale_oklch` 改用 `apply_kw`
- [x] 3.8 运行 `cargo test --test compile_test -- color` 和 `cargo test --test bs_spec` 确认 15/15 通过

## 4. 选择器规范化（css/selector.rs）

- [x] 4.1 ~ 跳过 — 设计决策 D5 例外：选择器规范化函数有复杂深度跟踪和回溯，保留 while 循环
- [x] 4.2 ~ 跳过 — 同上
- [x] 4.3 ~ 跳过 — 同上
- [x] 4.4 ~ 跳过 — 同上
- [x] 4.5 ~ 跳过 — 选择器规范化保留 while 状态机

## 5. 杂项函数式化

- [x] 5.1 `eval/rule.rs` 的 `nest_rule_in_children` 的 `for + push` 改为 `into_iter().fold`
- [x] 5.2 `eval/import.rs` 的 `eval_import` CSS 分支 `for + push` 改为 `urls.iter().map(...).collect()`
- [x] 5.3 `eval/builtin/selector.rs` 的 `selector-simple-selectors` 的 `for + push` 改为 `fold`
- [x] 5.4 `parse/ast_impl.rs` 的 `to_scss` If 分支 `for + push_str` 改为 `enumerate().fold`
- [x] 5.5 运行 `cargo test --test compile_test` 和 `cargo test --test ast_test` 确认通过

## 6. 全量验证

- [x] 6.1运行 `cargo test --test compile_test` 确认 43/43
- [x] 6.2运行 `cargo test --test stage_test` 确认 10/10
- [x] 6.3运行 `cargo test --test ast_test` 确认 8/8
- [x] 6.4运行 `cargo test --test common_test` 确认 5/5
- [x] 6.5运行 `cargo test --test interp_test` 确认 15/15
- [x] 6.6运行 `cargo test --test bs_spec` 确认 15/15
- [x] 6.7运行 `cargo test --test ep_full` 确认 121/121
- [x] 6.8运行 `cargo test --test default_config_test -- --test-threads=1` 确认 9/9
- [x] 6.9运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full` 确认 2902 基线不回归
- [x] 6.10运行 `cargo clippy --workspace` 确认无新 warning
