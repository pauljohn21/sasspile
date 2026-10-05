# Tasks

## 1. 插桩与根因确认

- [x] 1.1 在 `RuleBuilder::push` 的 Rule 分支添加 debug span 跟踪 `has_descendant_prefix` 决策（parent / child / result），用 `RUST_LOG="sasspile::rule_builder=trace"` 跑 `cargo test --test ep_classify_test -- --nocapture` 采集 dropdown.scss 的 trace 证据，确认根因 → 验证：trace 中 parent=`.el-dropdown__popper` child=`.el-dropdown__popper-selfdefine` has_prefix=`true`

## 2. 核心修复

- [x] 2.1 修改 `starts_with_compound_prefix` 的分隔符检测逻辑：将 `b".:#[>+~_-"` 改为仅 `--` 和 `__` 触发 compound 匹配（双字符精确匹配），保留 `.` `:` `#` `[` `>` `+` `~` 为 structural 分隔符 → 验证：`starts_with_compound_prefix(".el-popper", ".el-popper-selfdefine")` 返回 `false`；`starts_with_compound_prefix(".el-button", ".el-button--large")` 返回 `true`

## 3. 验证

- [x] 3.1 运行 `cargo test --test ep_classify_test -- --nocapture` 确认 14 个选择器嵌套文件 bug 分类缩小，EP 差异中 dropdown/menu/message/table/tag/timeline/transfer/tree/upload/popover/switch/form-item/message-box/overlay 均归为 "一致" 或 "autoprefixer only" → 验证：sasspile-bug 文件减少 10+

- [x] 3.2 运行 `cargo test --test compile_test --test reactor_test --test stage_test --test ast_test --test common_test --test interp_test --test bs_spec` 确认核心 202 测试 0 回归 → 验证：全部通过

- [x] 3.3 运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` 统计 sass-spec 通过率未下降 → 验证：通过率 >= 当前基线 7837/12133 (64.9%)

- [x] 3.4 运行 `cargo test --test ep_full --release -- --nocapture` 确认 EP 121/121 通过 → 验证：121/121

## 4. 清理

- [x] 4.1 移除临时 debug span（或降级为 `trace!`），保留生产级 span（rule_builder entry、eval_rule entry） → 验证：无 `error!` 级别临时日志
