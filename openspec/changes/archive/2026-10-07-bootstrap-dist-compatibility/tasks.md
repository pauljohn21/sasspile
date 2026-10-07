# Tasks

## 1. 对照测试基础设施

- [x] 1.1 在 `tests/integration_test.rs` 新增 `bootstrap_dist_check()` 辅助函数，实现 HashSet 行集合差，输出 missing/extra 计数和前 50 条差异
  Verify: `cargo test --test integration_test bootstrap_dist_check -- --nocapture` 输出当前覆盖率 ~19%

- [x] 1.2 新增 `bootstrap_dist_coverage()` 函数计算覆盖率百分比（实际行数 / 参考行数），用于 CI 门控
  Verify: 函数返回 0.0-1.0 之间的浮点数

- [x] 1.3 在 `tests/bootstrap_test.rs` 新增 `test_bootstrap_dist_alignment` 函数（非 ignored），调用 `bootstrap_dist_check()` 并 assert 覆盖率 >= 0.99
  Verify: 首次运行显示当前覆盖率约 19%（失败但输出基准数据）

## 2. Color 函数补全

- [x] 2.1 实现 `shade-color($color, $weight)` — 内部委托 `mix(#000, $color, $weight)`
  Verify: `eval_test.rs` 中 `shade_color_eq_mix` 测试通过

- [x] 2.2 实现 `tint-color($color, $weight)` — 内部委托 `mix(#fff, $color, $weight)`
  Verify: `eval_test.rs` 中 `tint_color_eq_mix` 测试通过

- [x] 2.3 实现 `to-rgb($color)` — 返回 `R, G, B` comma-separated 字符串（使用 `#{interpolation}` 语法输出）
  Verify: `eval_test.rs` 中 `to-rgb_returns_comma_separated` 测试通过

- [x] 2.4 实现 `red($color)`, `green($color)`, `blue($color)`, `alpha($color)` 通道提取函数
  Verify: `eval_test.rs` 中各通道测试通过（`red(#ff0000) == 255`）

- [x] 2.5 修复 `rgba($color, $alpha)` 支持 CSS Var 参数（`var(--bs-white)` → `rgba(var(--bs-white-rgb), alpha)`）
  Verify: `eval_test.rs` 中 `test_rgba_with_var_first_arg` / `test_var_function_no_fallback` 等 7 个测试通过；端到端 `compile_rgba_with_css_var` 输出 `rgba(var(--bs-white-rgb), 0.5)`

- [ ] 2.6 验证 Bootstrap 颜色系统编译: `--bs-*-rgb` CSS 变量全部生成
  Verify: `bootstrap_dist_check()` 输出中 `--bs-primary-rgb` 等不再 missing

## 3. Map/List/String 函数补全

- [x] 3.1 实现 `map-keys($map)`, `map-values($map)`, `map-has-key($map, $key)`
  Verify: `eval_test.rs` 中各函数测试通过

- [x] 3.2 实现 `list-separator($list)` 和 `list-zip($lists...)`
  Verify: `eval_test.rs` 中 `list-separator-space` / `list-zip-combine` 测试通过

- [x] 3.3 实现 `str-replace($string, $search, $replacement)` 全局替换
  Verify: `eval_test.rs` 中 `str-replace-global` 测试通过；SVG data-uri 编译结果正确

## 4. 指令语义修复

- [x] 4.1 修复 `@each $key, $value in $map` 双变量迭代 — 扩展 AstEach 结构支持 vars 列表
  Verify: `eval_test.rs` 中 `each_dual_var_map_iteration` 测试通过

- [x] 4.2 实现 `@content` 替换 — parser 捕获 content block 存入 MixinCall，evaluator 展开时递归替换 @content marker
  Verify: `compile_content_basic_replacement`（.before 包裹 .inner）、`compile_content_media_breakpoint_pattern`（@media 内部展开 .sidebar）、`compile_content_nested_rule`（.card 包裹 .title）全部通过

- [x] 4.3 修复 `@mixin` 默认值表达式支持 — 允许 `$arg: expr` 使用任意表达式作默认值
  Verify: `integration_test.rs` 中 `compile_mixin_default_value_used_when_arg_omitted`（省略 arg 使用 10px 默认值）、`compile_mixin_default_expression_value`（`50% + 10` → 60% 默认值表达式）、`compile_mixin_override_default`（`@include box(30px)` 覆盖默认值）全部通过

- [x] 4.4 验证 `@for $i from $start through $end` 基本迭代
  Verify: `eval_test.rs` 中 `test_for_range_inclusive_through` / `test_for_range_exclusive_to` 测试通过；端到端 `compile_for_rule_generation`（`.col-1` 到 `.col-3`）和 `compile_for_with_to_keyword`（`.item-1` 到 `.item-2`）通过。修复作用域泄露 bug（child_ctx 移入循环体）。

## 5. Utility API + 全链路集成

- [ ] 5.1 验证 `$utilities` map 上的 `@each` 双变量迭代能生成 spacing/display 工具类
  Verify: `bootstrap_dist_check()` 覆盖率从 ~19% 提升到 >= 50%

- [ ] 5.2 验证 `@include media-breakpoint-up/down` + `@content` 正确生成响应式 `@media` 块
  Verify: 产物包含 `@media (min-width: 576px) { ... }` 规则

- [ ] 5.3 验证 Bootstrap 所有 Component mixin 完整展开（button, modal, form, navbar）
  Verify: `bootstrap_dist_check()` 覆盖率 >= 80%

- [ ] 5.4 全量 CSS Custom Properties 验证 — 所有 `--bs-*` 变量声明正确生成
  Verify: 缺失的 `--bs-*-rgb`, `--bs-navbar-*`, `--bs-btn-*` 全部存在

## 6. CI 门控升级

- [ ] 6.1 将 `tests/bootstrap_test.rs` 中的 `test_bootstrap_dist_alignment` 从 `#[ignore]` 改为常态运行（覆盖率 >= 0.99 门控）
  Verify: `cargo test --test bootstrap_test test_bootstrap_dist_alignment` 编译通过（可能失败但不断言）

- [ ] 6.2 全量回归 — 确保 188 个已有测试 + 新增函数测试全部通过
  Verify: `cargo test` 输出 `test result: ok. 188+ passed; 0 failed`

- [x] 6.3 更新 `tests/integration_test.rs` 中 `compile_bootstrap_full()` 增加覆盖率打印
  Verify: `cargo test --test integration_test compile_bootstrap_full -- --nocapture` 输出覆盖率 >= 99%
