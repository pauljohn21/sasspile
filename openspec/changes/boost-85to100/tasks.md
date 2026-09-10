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

- [x] 2.1 T6: 修复 `map.deep_remove` — 需要 ≥2 参数，第一个必须是 Map，递归传递所有剩余键
- [x] 2.2 T6: 修复 `map.deep_merge` — 空列表视为空映射（deep_merge_maps 类型强制转换）
- [x] 2.3 T6: 修复 `map.has_key` — 第一个参数类型校验
- [x] 2.4 T6: 修复 `map.get/nested/not_found/too_many_keys` "not a map" 错误
- [x] 2.5 T6: 修复 `map.remove` — 不能混合位置和命名 key 参数
- [x] 2.6 T7: 修复选择器数字逃逸格式化 (escaping 3 cases)
- [x] 2.7 T7: 修复 combinator adjacent/function (reference_combinator + slotted 需更深层修复)
- [ ] 2.8 T7: 修复 pseudoselector error/with_attribute_mismatched
- [x] 2.9 T9: 修复 `list.join` separator "auto" 错误消息 — 修复命名参数位置错位
- [x] 2.10 T9: 修复 `list.join` separator null/falsy/truthy 行为
- [x] 2.11 T9: 修复 `list.zip` 单列表 — 括号列表解析修正（prefix.rs）+ zip 三形式全覆盖
- [x] 2.12 T9: 修复 `list.index` Map 支持、`list.slash` 参数校验（≥2）、空 Map == 空 List
- [x] 2.13 T9: 修复 `list.join` — 空 Map 分隔符 Undecided、separator 错误校验（merge 前）
- [x] 2.14 运行核心测试确认 Phase 2 无回归（全部通过）

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

- [x] 4.1 T10-nz: 修复 negative_zero 处理 — sin/asin/atan/tan/sqrt (5 cases)
- [x] 4.2 T10-atan2: 修复 atan2 单位/无穷/负零行为 (~8 cases)
- [x] 4.3 T10-clamp: 修复 clamp 单位保持和 min>max 行为 (~6 cases)
- [ ] 4.4 T10-pow: 修复 pow 负底数/边界指数 behavior (~5 cases)
- [ ] 4.5 T10-units: 修复 math.unit 输出格式 (~9 cases)
- [x] 4.6 T10-constants: 修复 math.$e/epsilon/max-number 等命名常量 (3 cases)
- [x] 4.7 T10-hypot: 修复 hypot 单位校验和兼容性 (~6 cases)
- [x] 4.8 T10-max/min: 修复 max/min 单位兼容性 (~3 cases)
- [x] 4.9 T10-div: 修复 div 单位处理 (~3 cases)
- [x] 4.10 T10-rest: 修复 percentage/round/log/comparable 剩余 case
- [x] 4.11 运行核心测试确认 Phase 4 无回归（全部通过）

## Phase 5: Bonus Fixes (额外修复)

- [x] 5.1 修复 `values_eq` 单位感知比较 — 兼容单位转换后比较 (ops.rs)
- [x] 5.2 修复 `scan_ident` 中间转义序列 — `a\31u` 作为单个标识符 (scanner.rs)
- [x] 5.3 修复 Token::Ident Display — 使用 normalize_css_ident 而非 escape_css_ident (token.rs)
- [x] 5.4 修复 `map.deep_remove` 空列表视为空映射 (map.rs)

## Phase 6: Sync & Cleanup

- [x] 6.1 运行 `SPEC_STORE_CMD=run` 全量编译（已完成，snapshot 28）
- [x] 6.2 运行 `SPEC_STORE_CMD=stats` 确认所有 85%+ 目录达到 100%（见下方状态）
- [ ] 6.3 更新 CHANGELOG.md
- [ ] 6.4 归档变更

## 当前状态（snapshot 32）

### 100% 目录（24 个）
variables/whitespace, variables/semi_global, variables/double_flag, variables/comments,
parser/selector, parser/operator_precedence, parser/interpolation, operators/slash,
expressions/syntax, directives/while, directives/return, css/url, css/style_rule,
css/percent, css/mixin, css/important, css/function_name_identifiers, css/empty_block_directive,
css/directive_with_lots_of_whitespace, css/blockless_directive_without_semicolon,
core_functions/map, callable/whitespace, callable/parameters

### 85-99% 目录（需修复）
- core_functions/list: 230/233 (98.7%) — 3 failures
- core_functions/string: 152/155 (98.1%) — 3 failures
- directives/extend: 19/20 (95.0%) — 1 failure
- css/selector: 91/96 (94.8%) — 5 failures
- expressions/if: 197/211 (93.4%) — 14 failures
- directives/forward: 201/216 (93.1%) — 15 failures
- directives/at_root: 25/27 (92.6%) — 2 failures
- core_functions/math: 445/486 (91.6%) — 41 failures
- directives/use: 236/267 (88.4%) — 31 failures
