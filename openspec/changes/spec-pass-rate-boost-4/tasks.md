## Phase 1 — 序列化空行修复

- [x] 1.1 ~~添加 `flatten_group` 字段~~ 改用 `group_id` + `is_same_origin` 混合方案
- [x] 1.2 `flatten_nodes` 返回 `Vec<(CssNode, usize)>` 同源 group_id
- [x] 1.3 `serialize_expanded` 中 group_id 相同或 `is_same_origin` 为 true 时不加空行
- [x] 1.4 声明穿插顺序由 `is_same_origin` 自动处理
- [ ] 1.5 修复注释在声明中的位置处理
- [x] 1.6 compile_test 43/43 通过
- [x] 1.7 css_diag 469→459 fail（interleaved 9→1）
- [x] 1.8 sass_spec_full 3078/5362 = 57%（+10，无回归）

## Phase 2 — 参数验证修复

- [x] 2.1 ~~`validate_single_number`~~ 已确认使用合并后 `args`，无需修改
- [x] 2.2 `str-index` 已使用 `merge_args`，确认无需修改
- [x] 2.3 `str-insert` 已使用 `merge_args`，确认无需修改
- [x] 2.4 `if` 函数命名参数合并——添加 `merge_meta_args` 在 `call_builtin` 中合并
- [ ] 2.5 修复 `rgba` 接受 3-4 number 参数
- [ ] 2.6 修复 `set-nth` 参数验证
- [x] 2.7 `coerce_number` 辅助函数已添加到 `math_helpers.rs`
- [x] 2.8 compile_test 43/43 通过
- [x] 2.9 sass_spec_full 确认无回归（3078 pass）

## Phase 3 — 内建函数补全

- [x] 3.1 `string.str-insert` 确认工作正常——`module_builtin_name` 映射正确
- [x] 3.2 `module_builtin_name` 映射已确认——所有 `STRING_NAMES` 条目可解析
- [ ] 3.3 修复 `utils.a` mixin/function 解析（callable 目录 24 fail）
- [ ] 3.4 修复 `str-index` / `str-slice` 参数类型强制转换
- [x] 3.5 Calc/字符串拼接运算符已在 `ops.rs` 中实现
- [x] 3.6 compile_test 43/43 通过
- [x] 3.7 sass_spec_full 3078 pass 确认无回归

## Phase 4 — 输出格式对齐

- [ ] 4.1 修复数值精度和格式化（infinity 单位等）
- [ ] 4.2 修复短/长 hex 颜色输出格式
- [ ] 4.3 修复选择器排序差异
- [ ] 4.4 修复 `@media` / `@supports` 合并规则边界
- [ ] 4.5 修复 `map.has-key` 返回值（extra_output 12 diff）
- [ ] 4.6 修复 `map.deep-remove` 不存在键处理
- [ ] 4.7 修复 `comparable` / `unit` 函数输出
- [ ] 4.8 运行 `cargo test --test cf_diag -- --nocapture` 验证 content_diff 下降
- [ ] 4.9 运行 sass_spec_full 验证整体通过率提升

## Phase 5 — plain CSS 错误检测

- [ ] 5.1 增强 `check_plain_css_node`：检测 `error/complex` 场景
- [ ] 5.2 增强 `check_plain_css_node`：检测 `error/compound` 场景
- [ ] 5.3 增强 `check_plain_css_node`：检测 `error/no_selector` 场景
- [ ] 5.4 修复 `@-moz-document` / `url-prefix` 解析
- [ ] 5.5 修复 `error/modifier/*` 系列检测
- [ ] 5.6 确保错误消息全部使用英文
- [ ] 5.7 运行 `cargo test --test css_diag -- --nocapture` 验证 expected_error_but_ok 下降
- [ ] 5.8 运行 sass_spec_full 验证 css 目录通过率提升

## Phase 6 — 模块系统修复

- [ ] 6.1 修复 `@use` module loop 检测（`loaded_modules` 更新时机）
- [ ] 6.2 修复 `@use with` 配置验证（consumed_config 传播）
- [ ] 6.3 修复 `@import` 文件冲突检测（partial/extension/index/import-only）
- [ ] 6.4 运行 `cargo test --test ep_full` 验证 121/121 无回归
- [ ] 6.5 运行 sass_spec_full 验证 directives/use 目录 fail 下降

## 验证

- [ ] 7.1 运行 `cargo test --test compile_test` — 43+ 通过
- [ ] 7.2 运行 `cargo test --test stage_test` — 10/10 通过
- [ ] 7.3 运行 `cargo test --test ep_full` — 121/121 通过
- [ ] 7.4 运行 `cargo test --test default_config_test -- --test-threads=1` — 9/9 通过
- [ ] 7.5 运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` — 验证总体通过率从 57% 提升
- [ ] 7.6 运行 `cargo clippy --workspace` — 无新 warning
- [ ] 7.7 codegraph sync + commit（等用户确认后推送）
