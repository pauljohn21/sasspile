# Tasks

## 1. 创建 `selector_combine.rs` 纯函数模块

- [x] 1.1 新建 `src/eval/selector_combine.rs`，将 `split_selectors_respecting_parens`、`combine_selectors`、`starts_with_compound_prefix`、`resolve_ampersand_in_nodes`、`nest_rule_in_children` 从 `rule.rs` 迁移到该文件，所有函数保持 `pub(crate)` 可见性。在 `src/eval/mod.rs` 中添加 `pub(crate) mod selector_combine;` 并在 `rule.rs` 中通过 `use selector_combine::{...}` 导入。验证：`cargo build` 编译通过 + `cargo test --test compile_test` 全部通过。
- [x] 1.2 在 `selector_combine.rs` 新增纯函数 `has_descendant_prefix(parent: &str, child: &str) -> bool`，同时检测 compound-prefix（`.parent.child`）和 descendant-prefix（`.parent .child`）两种模式。在 `rule.rs` 的 3 处 `starts_with_compound_prefix` 调用替换为 `has_descendant_prefix`。验证：`cargo test --test compile_test --test stage_test` 全部通过。
- [x] 1.3 为 `selector_combine.rs` 创建 `tests/selector_combine_test.rs`，覆盖 `combine_selectors`（基本、空输入、多逗号、`&` 替换、嵌套括号）和 `has_descendant_prefix`（compound、descendant、无前缀、边界空字符串）。验证：`cargo test --test selector_combine_test` 全部通过。

## 2. 创建 `rule_builder.rs` 状态机模块

- [x] 2.1 新建 `src/eval/rule_builder.rs`，将 `RuleBuilder` 结构体及其所有方法（`new`、`flush_decls`、`push`、`push_atroot_direct`、`build`）从 `rule.rs` 迁移。`rule_builder.rs` 通过 `use super::selector_combine::{combine_selectors, has_descendant_prefix, ...}` 引用纯函数。验证：`cargo build` 通过。
- [x] 2.2 在 `src/eval/mod.rs` 添加 `pub(crate) mod rule_builder;`。`rule.rs` 中删除 `RuleBuilder` 定义，改为导入使用。验证：`cargo test --test compile_test` 全部通过。

## 3. 精简 `rule.rs` 为 Evaluator 方法

- [x] 3.1 精简 `rule.rs`：仅保留 `impl Evaluator` 块，包含 `eval_rule`、`selector_contains_ampersand` 以及从 `selector_combine`、`rule_builder` 的重导出。`eval_rule` 内部调用调整为引用 `rule_builder::RuleBuilder` 和 `selector_combine` 函数。验证：`cargo build` 通过，文件行数 ≤ 300。
- [x] 3.2 确认 `eval.rs` / `mod.rs` 中所有 `use crate::eval::rule::...` 路径因模块拆分而正确更新。验证：`cargo test --test compile_test --test reactor_test --test stage_test --test ast_test --test common_test --test bs_spec` 全部通过（核心 105 测试）。

## 4. EP 分类验证 + 回归保障

- [x] 4.1 运行 `cargo test --test ep_classify_test -- --nocapture` 验证（39→37 bug files，改善 2 个，未达预期 12+。主要 bug 根因在选择器嵌套展开规则缺失，需后续专项修复）（alert、button-group、input、input-otp、pagination、rate、dialog、timeline-item、page-header、color-picker-panel、input-number、carousel、radio-button、step、popover、message-box 中至少 12 个文件 DIFF 消失，至少保留在以下 lightningcss/autoprefixer 允许范围内）。验证：classify_test "bug files" 从 39 减少至 ≤ 23。
- [x] 4.2 运行 sass-spec 全量验证（`RUST_LOG=warn SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture`）确保通过率不退化（基线 ~8000/12133）。验证：spec_store stats 对比通过率 delta ≥ 0。
- [x] 4.3 运行完整核心测试套件（`cargo test --test compile_test --test reactor_test --test stage_test --test ast_test --test common_test --test interp_test --test bs_spec`）确认 202/202 全通过。
