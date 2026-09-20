# EP Consistency Boost — Tasks

## Phase 1: Parent Selector `&` 展开（Category A，P0）— ✅ COMPLETE

- [x] **T1.1**: 添加 `selector_contains_ampersand` 辅助函数 — 递归检测 CssNode 选择器中的 `&`
- [x] **T1.2**: 修改 `RuleBuilder::push` AtRoot 分支 — 检测 `&` 后走 `nest_rule_in_children`
- [x] **T1.3**: 回归测试 — `avatar.scss` ✅（`&--circle`→`.el-avatar--circle`）
- [x] **T1.4**: 回归测试 — `breadcrumb.scss` ✅（`.el-breadcrumb::before` via utils-clearfix mixin）
- [x] **T1.5**: 回归测试 — `badge.scss` ✅
- [x] **T1.6**: 回归测试 — `button.scss` 部分（rgba→transparent 待 T3.3）
- [x] **T1.7**: 回归测试 — `container.scss` ✅
- [x] **T1.8**: 核心测试 — 130/130 全通过 ✅
- [x] **T1.9**: `nest_rule_in_children` 递归处理 Rule/AtRule/AtRoot 子节点 (新增, commit 250c240)

## Phase 2: SCSS 嵌套函数求值（Category C，P1）— ✅ PARTIAL

- [x] **T2.1**: 诊断 `display.scss` — 定位 `string.unquote(map.get(...))` 未求值节点 (commit: eval_at_rule fix)
  - 根因: `eval_at_rule` 先用 `eval_nodes` 处理 body（触发 `Rule` 的 `enter_scope/exit_scope`），导致 mixin 局部变量 `$map` 丢失
  - `eval_interp_str` 在 `eval_simple_expr` 失败时回退到原始文本（不报错），输出 `string.unquote(map.get($map, $key))` 字面量
  - 诊断方法: 在 `eval_interp_str` 错误分支写入 `/tmp/interp_eval_error.txt`
  - 修复: 将参数求值移到 body 处理之前，使用原始 `env` 而非 `new_env`
- [x] **T2.2**: 修复 builtin 嵌套函数分派 — `string.unquote` 接受函数调用结果作为参数 (同 T2.1)
- [ ] **T2.3**: 诊断 `input-otp.scss` / `select-dropdown-v2.scss` — `getCssVar` 嵌套未展开
- [x] **T2.4**: ~~修复 getCssVar 在 calc/attr 表达式中的递归展开~~ — 通用化为 calc() 内部 Sass 函数求值（commit: var/calc EP fix）
  - `src/eval/value/mod.rs`: `try_eval_calc_inner_functions()` 扫描 calc 字符串中 `ident(...)` 模式，识别用户函数并求值替换
  - `calc(getCssVar("index","normal") - 1)` → `calc(var(--index, normal) - 1)` ✅
- [x] **T2.5**: 回归测试 — display.scss ✅ 无 diff（验证通过 T2.1/T2.2 修复）

- [x] **T2.6**: var() 回退值 Sass 表达求值 — `var(--x, map.get($map, a))` 正确展开 (commit: var/calc EP fix)
  - `src/parse/params.rs`: 新增 `parse_args_prefix()` + `parse_args_inner(stop_at_rparen: bool)`
  - `src/parse/expr/literals.rs`: var() 分支尝试结构化参数解析，成功返回 `Value::Call`
  - `var(--x, map.get($map, a))` → `var(--x, 1)` ✅

## Phase 3: CSS 格式化对齐（Category B，P2）— ✅ PARTIAL

- [x] **T3.2**: CSS 函数名大小写规范化 — `scalex`→`scaleX`, `rotatez`→`rotateZ`, `translatex`→`translateX` (commit 250c240)
  - 修复文件: collapse-transition, divider, icon(部分), image-viewer, badge 等
- [x] **T3.4**: Keyframes `\%` → `%` — 跳过 keyframes 子节点的 dedup_compound_simples (commit 250c240)
  - 修复文件: dialog, drawer, icon, message-box 等
- [ ] **T3.1**: 颜色规范化 — white/black → hex（需区分 sass-spec 兼容性 vs EP 需求）
- [ ] **T3.3**: `rgba(0,0,0,0)` → `transparent` 转换
- [ ] **T3.5**: 回归测试 — 编译 `base.scss`、`icon.scss`、`menu.scss`

## Phase 4: LightningCSS 兼容（P3，依赖 B/C）

- [ ] **T4.1**: 复测 lightningcss FAIL 列表（期望部分随 B/C 修复而消失）
- [ ] **T4.2**: 修复 EmptySelector 检测（step.scss、popper.scss、date-picker-panel.scss）
- [ ] **T4.3**: 修复 PseudoElementExpectedIdent（table.scss）

## Phase 5: 颜色规范化 + rgba 优化（P2 续）

- [ ] **T5.1**: `rgba(0,0,0,0)` → `transparent` — 在 `Color::display` 检测零-alpha 全零 RGB
- [ ] **T5.2**: named color → hex — `reverse_lookup_named_color` 退役或走 hex-only 输出
- [ ] **T5.3**: `var( ` → `var(` 空格清理（`eval_property_name` 插值字符串内归一化）
- [x] **T5.4**: `getCssVar("x","d") - 1` 在 calc 中求值 — 已由 T2.4 通用方案覆盖

## 验收标准

- [x] EP 一致性 ≥ 45/121 (37.2%) — 当前值
- [ ] EP 一致性 ≥ 80/121 (66%) — 目标
- [x] 核心测试 130/130 全通过（46+14+8+8+5+15+15+8+1=120 + bs_spec 10）
- [x] sass-spec 通过率 ≥ 65.7% — 7979/12133 (65.7%), +2 vs baseline 7977
