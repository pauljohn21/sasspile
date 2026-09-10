# boost-85to100 — Tasks

## Phase 1: Quick Wins (~16 cases, 直接修复)

- [x] 1.1 T1: 修复 `parse_use` 注释解析 — 在关键位置改用 `skip_ws_and_comments()`
- [x] 1.2 T1: 修复 `parse_forward` 注释解析 — 同上（含 `parse_config` + `parse_member_list`）
- [x] 1.3 T2: 修复 CSS 序列化器空字符串属性抑制 — value 为 `""` 时跳过属性
- [x] 1.4 T3: 修复 list equality — `[] == ()` 返回 false (增加 bracketed 检查)
- [x] 1.5 T5: 修复 CSS @import 在规则内的提升 — hoist_css_imports 递归提取 Rule children 中的 @import
- [ ] 1.6 T8-partial: 修复 `@if css/alone/argument` 的 `?` lex 错误（需 lexer 扩展）
- [ ] 1.7 T8-partial: 修复 `@if syntax/trailing_semi` 尾随分号解析
- [x] 1.8 运行核心测试确认 Phase 1 无回归（全部通过）

## Phase 2: Medium Effort (~29 cases)

- [ ] 2.1 T6: 修复 `map.deep_remove` 未实现 (4 cases: too_few_args, type, extra_keys, not_a_map)
- [ ] 2.2 T6: 修复 `map.deep_merge` 空 map 处理 (1 case: empty/second)
- [ ] 2.3 T6: 修复 `map.has_key` 类型错误消息 (1 case: error/type/map)
- [ ] 2.4 T6: 修复 `map.get/nested/not_found/too_many_keys` "not a map" 错误
- [ ] 2.5 T6: 修复 `map.remove` positional_and_named 错误消息
- [ ] 2.6 T7: 修复选择器数字逃逸格式化 (escaping 3 cases)
- [ ] 2.7 T7: 修复 combinator adjacent/function, reference_combinator, slotted
- [ ] 2.8 T7: 修复 pseudoselector error/with_attribute_mismatched
- [ ] 2.9 T9: 修复 `list.join` separator "auto" 错误消息
- [ ] 2.10 T9: 修复 `list.join` separator null/falsy/truthy 行为
- [ ] 2.11 T9: 修复 `list.zip` 单列表 bracketed 行为
- [ ] 2.12 T9: 修复 `list.index` map 行为、`list.slash`、`list.utils`
- [ ] 2.13 T9: 修复 `list.join` empty map 的 slash 和 space 行为
- [ ] 2.14 运行核心测试确认 Phase 2 无回归

## Phase 3: Substantial Effort (~76 cases)

- [ ] 3.1 T4: 修复传递 @extend 穿越 `:is()` 伪选择器
- [ ] 3.2 T8-error: 修复 `@if error/*` 9 个 and/or/not 错误消息差异
- [ ] 3.3 T8-raw: 修复 `@if raw/*` 3 个原始逻辑操作符解析
- [ ] 3.4 T11-partial: 修复 `@forward` member import precedence 和 override (8 DIFFs)
- [ ] 3.5 T11-member-as: 修复 `@forward ... as` 分隔符处理 (4 ERRs)
- [ ] 3.6 T12: 修复 `@use` CSS 排序 — use + import 混合顺序 (5 DIFFs)
- [ ] 3.7 T12: 修复 `@use/member/namespaced` 变量赋值行为
- [ ] 3.8 运行核心测试确认 Phase 3 无回归

## Phase 4: Heavy Effort — Math Functions (73 cases)

- [ ] 4.1 T10-nz: 修复 negative_zero 处理 — sin/asin/atan/tan/sqrt (5 cases)
- [ ] 4.2 T10-atan2: 修复 atan2 单位/无穷/负零行为 (~8 cases)
- [ ] 4.3 T10-clamp: 修复 clamp 单位保持和 min>max 行为 (~6 cases)
- [ ] 4.4 T10-pow: 修复 pow 负底数/边界指数 behavior (~5 cases)
- [ ] 4.5 T10-units: 修复 math.unit 输出格式 (~9 cases)
- [ ] 4.6 T10-constants: 修复 math.$e/epsilon/max-number 等命名常量 (3 cases)
- [ ] 4.7 T10-hypot: 修复 hypot 单位校验和兼容性 (~6 cases)
- [ ] 4.8 T10-max/min: 修复 max/min 单位兼容性 (~3 cases)
- [ ] 4.9 T10-div: 修复 div 单位处理 (~3 cases)
- [ ] 4.10 T10-rest: 修复 percentage/round/log/comparable 剩余 case
- [ ] 4.11 运行核心测试确认 Phase 4 无回归

## Phase 5: Sync & Cleanup

- [ ] 5.1 运行 `SPEC_STORE_CMD=run` 全量编译
- [ ] 5.2 运行 `SPEC_STORE_CMD=stats` 确认所有 85%+ 目录达到 100%
- [ ] 5.3 更新 CHANGELOG.md
- [ ] 5.4 归档变更
