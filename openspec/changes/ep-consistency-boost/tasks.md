# EP Consistency Boost — Tasks

## Phase 1: Parent Selector `&` 展开（Category A，P0）

- [ ] **T1.1**: 添加 `selector_contains_ampersand` 辅助函数 — 递归检测 CssNode 选择器中的 `&`
- [ ] **T1.2**: 修改 `RuleBuilder::push` AtRoot 分支 — 检测 `&` 后走 `nest_rule_in_children`
- [ ] **T1.3**: 回归测试 — 编译 `avatar.scss` 验证 `.el-avatar--circle` / `.el-avatar--square`
- [ ] **T1.4**: 回归测试 — 编译 `breadcrumb.scss` 验证 `.el-breadcrumb::before`
- [ ] **T1.5**: 回归测试 — 编译 `badge.scss` 验证 `.el-badge__content.is-fixed`
- [ ] **T1.6**: 回归测试 — 编译 `button.scss` 验证 `.el-button.is-plain`
- [ ] **T1.7**: 回归测试 — 编译 `container.scss` 验证 `.el-container.is-vertical`
- [ ] **T1.8**: 核心测试 — 确认 sass-spec + 241 核心测试无退化

## Phase 2: SCSS 嵌套函数求值（Category C，P1）

- [ ] **T2.1**: 诊断 `display.scss` — 定位 `string.unquote(map.get(...))` 未求值节点
- [ ] **T2.2**: 修复 builtin 嵌套函数分派 — `string.unquote` 接受函数调用结果作为参数
- [ ] **T2.3**: 诊断 `input-otp.scss` / `select-dropdown-v2.scss` — `getCssVar` 嵌套未展开
- [ ] **T2.4**: 修复 getCssVar 在 calc/attr 表达式中的递归展开
- [ ] **T2.5**: 回归测试 — display.scss、input-otp.scss、select-dropdown-v2.scss

## Phase 3: CSS 格式化对齐（Category B，P2）

- [ ] **T3.1**: 颜色规范化 — white/black/red/green/blue/transparent 映射为 dart-sass hex 输出
- [ ] **T3.2**: CSS 函数名大小写规范化 — rotateZ/scaleX/translateX 等
- [ ] **T3.3**: `rgba(0,0,0,0)` → `transparent` 转换
- [ ] **T3.4**: Keyframes `\%` → `%` 取消转义
- [ ] **T3.5**: 回归测试 — 编译 `base.scss`、`icon.scss`、`menu.scss`

## Phase 4: LightningCSS 兼容（P3，依赖 B/C）

- [ ] **T4.1**: 复测 lightningcss FAIL 列表（期望部分随 B/C 修复而消失）
- [ ] **T4.2**: 修复 EmptySelector 检测（step.scss、popper.scss、date-picker-panel.scss）
- [ ] **T4.3**: 修复 PseudoElementExpectedIdent（table.scss）

## 验收标准

- [ ] EP 一致性 ≥ 80/121 (66%)
- [ ] 核心测试 ≥ 241/241
- [ ] sass-spec 通过率 ≥ 65.7%
