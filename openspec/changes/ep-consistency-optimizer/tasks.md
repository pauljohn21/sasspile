## 1. 诊断基础设施

- [ ] 1.1 创建 `tests/ep_classify_test.rs` 诊断测试文件，实现 autoprefixer 属性剥离 + DIFF 分类报告
- [ ] 1.2 验证分类报告输出正确区分 sasspile-bug / autoprefixer-only / lightningcss-artifact / EP-技术改版
- [ ] 1.3 对每个 DIFF 文件输出精确的 diff 上下文（第一处差异位置 + 前后 30 字符）

## 2. BEM 嵌套 `&` 作用域修复（A 类，+8 files）

- [ ] 2.1 在 `src/eval/rule.rs` 的 `RuleBuilder` 入口添加 `#[instrument]` span 插桩，追踪 `self.selector` 在 `push_atroot_direct` 调用时的值
- [ ] 2.2 采集 anchor.scss 的 tracing trace，确认嵌套 AtRootDirect 场景中 `self.selector` 的实际值（当前预期为错误的外层 selector）
- [ ] 2.3 实现修复：在 `push_atroot_direct` 中，当 AtRootDirect 嵌套 AtRootDirect 时，使用内层 selector + 外层 parent 组合（而非仅用 `self.selector`）
- [ ] 2.4 验证 anchor.scss 输出为 `.el-anchor.el-anchor--horizontal .el-anchor__list .el-anchor__item`（三层完整）
- [ ] 2.5 验证 descriptions.scss、color-picker-panel.scss、popover.scss、select.scss、select-v2.scss、step.scss、color-picker.scss 全部修复
- [ ] 2.6 全量运行 compile_test + stage_test + ast_test + common_test + bs_spec + sass-spec 验证无回归
- [ ] 2.7 跑 `cargo test --test ep_full` 确认无新增编译失败

## 3. @keyframes 空步骤剥离（B 类，+3 files）

- [ ] 3.1 新建 `src/css/keyframes.rs` 模块，实现 `strip_empty_keyframe_steps(nodes: &mut Vec<CssNode>)` 函数
- [ ] 3.2 实现首尾空步骤检测逻辑：`0%`/`from` 空、`100%`/`to` 空 → 移除；中间空步骤保留
- [ ] 3.3 在 `Serializer::serialize_expanded` 和 `Serializer::serialize_compressed` 入口调用空步骤剥离
- [ ] 3.4 验证 dialog.scss、drawer.scss、message-box.scss 的 `v-modal-in`/`v-modal-out` 空步骤被移除
- [ ] 3.5 全量运行核心测试 + sass-spec 验证无回归（正常 keyframes 不受影响）

## 4. Selector/Value Semantic Fixes（C 类）

- [ ] 4.1 诊断 button.scss 变量值差异根因（`--el-button-disabled-color` vs `--el-button-text-color`）——确认是变量作用域 bug 还是 EP 版本差异
- [ ] 4.2 诊断 form-item.scss `margin-bottom: 0` 位置错误——确认是否因 AtRootDirect 嵌套顺序引起
- [ ] 4.3 诊断 input.scss / tour.scss `null` fallback 格式——确认 var() null → 空字符串 转换逻辑
- [ ] 4.4 诊断 table.scss / table-column.scss `-moz-appearance` 差异——确认为 autoprefixer 产物
- [ ] 4.5 诊断 date-picker-panel.scss 首字符差异（`.` 开头 vs 无 `.`）
- [ ] 4.6 诊断 pagination.scss `user-select:none` 位置——确认为 autoprefixer 注入
- [ ] 4.7 诊断 splitter.scss 选择器丢失
- [ ] 4.8 诊断 image-viewer.scss 属性顺序差异
- [ ] 4.9 诊断 table-v2.scss 属性分组差异
- [ ] 4.10 诊断 empty.scss 属性位置
- [ ] 4.11 诊断 input-tag.scss `transform: translate(0,0)` vs `translate(0)`——lightningcss 化简产物
- [ ] 4.12 诊断 time-picker.scss / time-select.scss `-moz-appearance` 前缀

## 5. C 类批量修复

- [ ] 5.1 实现 var() null fallback 序列化修复（`null` → 空字符串）
- [ ] 5.2 修复 form-item margin-bottom 位置（如确认为 AtRootDirect 顺序问题）
- [ ] 5.3 修复其他已确认的 semantic bug
- [ ] 5.4 全量核心测试 + sass-spec 验证无回归

## 6. EP Normalized Test 管线增强

- [ ] 6.1 在 `tests/ep_normalized_test.rs` 中增加 `strip_autoprefixer_properties` 函数
- [ ] 6.2 实现规则：移除 `-webkit-user-select`、`-webkit-appearance`、`-moz-user-select`、`-ms-user-select`、`-webkit-inner-spin-button`、`-webkit-outer-spin-button`、`-webkit-tap-highlight-color`、`-webkit-box-orient`、`-webkit-line-clamp`、`backface-visibility`
- [ ] 6.3 更新 `normalize_css` 函数：minify 后调用 autoprefixer stripping
- [ ] 6.4 运行 `cargo test --test ep_normalized_test -- --nocapture` 确认 autoprefixer-only 文件（7个）不再报 DIFF
- [ ] 6.5 确认剩余 DIFF 文件均为真正的 sasspile bug

## 7. 最终验证

- [ ] 7.1 运行全量核心测试 `cargo test --test compile_test --test stage_test --test ast_test --test common_test --test bs_spec` 确认全部通过
- [ ] 7.2 运行 `cargo test --test ep_full` 确认 121/121 编译成功
- [ ] 7.3 运行 `cargo test --test ep_classify_test` 统计最终 DIFF 数
- [ ] 7.4 确认 A 类 8 files + B 类 3 files 已通过修复解决
- [ ] 7.5 统计最终 EP 语义一致性目标：~111/121 (91.7%)
