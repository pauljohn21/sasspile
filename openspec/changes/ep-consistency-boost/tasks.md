# EP Consistency Boost — Tasks

## Phase 1: Parent Selector `&` 展开（P0）— ✅ COMPLETE

- [x] T1.1: selector_contains_ampersand 辅助函数
- [x] T1.2: RuleBuilder::push AtRoot 分支检测 & 走 nest_rule_in_children
- [x] T1.3: nest_rule_in_children 递归处理 Rule/AtRule/AtRoot 子节点
- [x] T1.4: 回归测试 — avatar, breadcrumb, badge, button, container 等
- [x] T1.5: 核心测试 130/130 全通过

## Phase 2: SCSS 嵌套函数求值（P1）— ✅ COMPLETE

- [x] T2.1: eval_at_rule 参数求值前移修复 display.scss
- [x] T2.2: calc() 内部 Sass 函数通用求值（try_eval_calc_inner_functions）
- [x] T2.3: var() 回退值 Sass 表达式求值（parse_args_prefix）
- [x] T2.4: 回归测试 — display.scss ✅

## Phase 3: CSS 格式化对齐（P2）— ✅ PARTIAL

- [x] T3.1: color.mix RgbPercent 强制输出（base.scss, var.scss 修复）
- [x] T3.2: `#{& + '-x'}` 插值展开（overlay.scss 修复）
- [ ] T3.3: @extend %placeholder 分组（image.scss）

## Phase 4: AtRootDirect — mixin @at-root 源码位置（D3）— ✅ COMPLETE

- [x] T4.1: CssNode::AtRootDirect 变体新增（src/css/node.rs）
- [x] T4.2: exec_mixin 求值后 AtRoot → AtRootDirect 替换（src/eval/mixin.rs）
- [x] T4.3: RuleBuilder::push 独立处理 AtRootDirect（src/eval/rule.rs）
- [x] T4.4: 全链路适配 — serialize/hoist/extend/meta_ops
- [x] T4.5: 测试 — test_mixin_order.rs
- [x] T4.6: 验证 backtop.scss mixin @at-root 源码顺序 ✅
- [x] T4.7: sass-spec 影响确认（-84 case，可接受）

## Phase 5: 字面 `&` 条件展开（D4）— ✅ COMPLETE

- [x] T5.1: RuleBuilder::push AtRootDirect 分支增加 `&` 检测
- [x] T5.2: 含 `&` 走 combine_selectors；不含 `&` 直接 push
- [x] T5.3: 测试 — test_ep_amp_fix.rs（5 文件：segmented/timeline/pagination/tree/popover）
- [x] T5.4: clippy cleanup — extend.rs/hoist.rs unwrap → expect
- [x] T5.5: 核心测试 130/130 + sass-spec 7893 稳定
- [x] T5.6: EP 73/121 IDENTICAL（60.3%）

## Phase 6: @extend %placeholder 分组（PENDING）

- [ ] T6.1: 收集所有 `%placeholder { ... }` 的定义（ModuleExports 扩展）
- [ ] T6.2: apply_extends 逻辑增强 — 将 placeholder 声明复制到各 extender
- [ ] T6.3: 输出阶段检测多 extender 共享同一 placeholder → 生成组合选择器
- [ ] T6.4: 回归测试 — image.scss, descriptions.scss, form-item.scss
- [ ] T6.5: sass-spec 回归检验

## 验收标准

- [x] EP 一致性 ≥ 45/121 (37.2%)
- [x] EP 一致性 ≥ 61/121 (50.4%)
- [x] EP 一致性 ≥ 73/121 (60.3%) — 当前值
- [ ] EP 一致性 ≥ 80/121 (66%) — 短期目标
- [ ] EP 一致性 ≥ 100/121 (83%) — 中期目标
- [x] 核心测试 130/130 全通过
- [x] sass-spec 通过率 ≥ 65%（7893/12133）
