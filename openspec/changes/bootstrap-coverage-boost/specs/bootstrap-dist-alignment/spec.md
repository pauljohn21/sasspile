# Spec Delta

## ADDED Requirements

### Requirement: 序列化缩进正确性（B6/B7 修复）
Expanded 模式输出 SHALL 使用 2 空格缩进（depth=1 → 2 spaces, depth=2 → 4 spaces）。选择器与 `{` 之间 SHALL 有且仅有 1 个空格。

#### Scenario: Decl 缩进正确
- **WHEN** 嵌套规则中的 decl 处于 depth=1
- **THEN** decl 缩进 SHALL 为 2 个空格（`  prop: val;`）
- **WHEN** depth=2
- **THEN** decl 缩进 SHALL 为 4 个空格（`    prop: val;`）

#### Scenario: 选择器空格正确
- **WHEN** 输出 `.a` 选择器块
- **THEN** 输出 SHALL 为 `.a {` 而非 `.a{`
- **WHEN** 输出 `@media (min-width: 576px) {`
- **THEN** 输出 SHALL 为 `@media (min-width: 576px) {`（`{` 前有空格）

### Requirement: `!important` 标记处理（B8 修复）
SHALL 正确解析并输出 `!important` 标记的声明。`prop: val !important;` SHALL 原样保留到输出 CSS。

#### Scenario: !important decl 保留
- **WHEN** SCSS 源中含 `color: red !important;`
- **THEN** 编译输出 SHALL 为 `color: red !important;`（保留标记及空格）

#### Scenario: !important 在嵌套规则中
- **WHEN** SCSS 规则嵌套中含 `margin: 0 !important;`
- **THEN** 输出 SHALL 在正确缩进位置输出 `margin: 0 !important;`

### Requirement: 厂家前缀生成（B9 修复）
调用 auto-prefixer mixin 时 SHALL 生成厂家前缀变体声明（`-webkit-`, `-moz-`, `-ms-`）。

#### Scenario: transition 前缀
- **WHEN** SCSS 调用 `@include transition-transform` 或类似 mixin
- **THEN** 输出 SHALL 同时包含 `-webkit-transition`, `-ms-transition`, `transition` 声明（前缀顺序遵循 dart-sass 惯例）

#### Scenario: transform 前缀
- **WHEN** mixin 生成 `transform: translateX(0)`
- **THEN** 输出 SHALL 包含 `-webkit-transform`, `-ms-transform`, `transform` 三个变体

### Requirement: 覆盖率目标 ≥ 99%
编译 coverage SHALL 达到 99%+（≤ 53 行不包括注释的差异）。

#### Scenario: 覆盖率达到 99%
- **WHEN** 运行 `bootstrap_dist_check()`
- **THEN** `missing_count ≤ 53`（总参考行 5342 的 1%）

#### Scenario: 覆盖率渐进提升
- **WHEN** 修复 B6/B7（缩进+空格）
- **THEN** coverage SHALL 从 1.25% 提升至 ~50%
- **WHEN** 修复 B8（!important）
- **THEN** coverage SHALL 再提升 ~12%
- **WHEN** 修复 B9-B11（前缀+mixin）
- **THEN** coverage SHALL 最终达到 ≥ 99%
