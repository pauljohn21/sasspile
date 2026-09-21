# EP Consistency Boost — Tasks

## Phase 1: Parent Selector `&` 展开（P0）— ✅ COMPLETE

- [x] T1.1: selector_contains_ampersand 辅助函数
- [x] T1.2: RuleBuilder::push AtRoot 分支检测 & 走 nest_rule_in_children
- [x] T1.3: nest_rule_in_children 递归处理 Rule/AtRule/AtRoot 子节点
- [x] T1.4: 回归测试 — avatar, breadcrumb, badge, button, container 等
- [x] T1.5: 核心测试 129/129 全通过

## Phase 2: SCSS 嵌套函数求值（P1）— ✅ COMPLETE

- [x] T2.1: eval_at_rule 参数求值前移修复 display.scss
- [x] T2.2: calc() 内部 Sass 函数通用求值（try_eval_calc_inner_functions）
- [x] T2.3: var() 回退值 Sass 表达式求值（parse_args_prefix）
- [x] T2.4: 回归测试 — display.scss ✅

## Phase 3: CSS 格式化对齐（P2）— ✅ PARTIAL

- [x] T3.1: color.mix RgbPercent 强制输出（base.scss, var.scss 修复）
- [x] T3.2: `#{& + '-x'}` 插值展开（overlay.scss 修复）
- [ ] T3.3: @extend %placeholder 分组（image.scss）

## Phase 4: 选择器结构/排序优化（P3）— 🔄 IN PROGRESS

### 4.1 高相似度文件快速修复

- [ ] T4.1.1: 诊断 backtop.scss (85%) — 规则排序差异
- [ ] T4.1.2: 诊断 segmented.scss (83%)
- [ ] T4.1.3: 诊断 tree.scss (69%)
- [ ] T4.1.4: 诊断 popover.scss (63%) / pagination.scss (62%)

### 4.2 @extend 分组

- [ ] T4.2.1: 理解 image.scss 中 `@extend %placeholder` 的dart-sass 行为
- [ ] T4.2.2: 实现 selector-ast 层面的 @extend 合并
- [ ] T4.2.3: 回归测试 — image.scss

### 4.3 CAT1 @at-root/BEM 深度修复

- [ ] T4.3.1: 诊断 popper.scss EmptySelector 根因
- [ ] T4.3.2: 诊断 cascader/select 规则排序差异
- [ ] T4.3.3: 实施 mixin 展开顺序对齐

### 4.4 EmptySelector/编译失败

- [ ] T4.4.1: 诊断 dialog.scss / drawer.scss @keyframes calc 空块
- [ ] T4.4.2: 实施 keyframes 内 calc() 特殊处理

## 验收标准

- [x] EP 一致性 ≥ 45/121 (37.2%)
- [x] EP 一致性 ≥ 61/121 (50.4%) — 当前值
- [ ] EP 一致性 ≥ 80/121 (66%) — 短期目标
- [ ] EP 一致性 ≥ 100/121 (83%) — 中期目标
- [x] 核心测试 129/129 全通过
- [x] sass-spec 通过率 ≥ 65.7%
