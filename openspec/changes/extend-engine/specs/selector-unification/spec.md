## ADDED Requirements

### Requirement: Compound 选择器统一

系统 SHALL 实现 compound 选择器内的 partial 替换算法，当 extender 的最后一个 compound 与 extendee 在原选择器中匹配时，替换匹配部分并保留其余 simple selector。

#### Scenario: 简单 compound 统一

- **GIVEN** extender = `.bar`，extendee = `.foo`，规则选择器 = `.foo`
- **WHEN** 系统应用 extend
- **THEN** 结果 MUST 为 `.foo, .bar`

#### Scenario: compound 内 partial 替换

- **GIVEN** extender = `.a .b`，extendee = `.e`，规则选择器 = `.e.f`
- **WHEN** 系统在 compound `.e.f` 中找到 `.e` 并用 extender 的最后 compound `.b` 替换
- **THEN** 结果 MUST 包含 `.a .b.f`（`.f` 保留，`.e` 被替换为 `.b`，前缀 `.a` 以 descendant combinator 拼接）

#### Scenario: 多重 compound 统一

- **GIVEN** `.a .b {@extend .e}` 和 `.c .d {@extend .f}`，规则选择器 = `.e.f`
- **WHEN** 系统依次应用两个 extends
- **THEN** 结果 MUST 包含 `.e.f`、`.a .f.b`、`.c .e.d`、`.a .c .b.d`、`.c .a .b.d`

### Requirement: Complex 选择器 Weave 交织

系统 SHALL 实现 complex 选择器的 weave 算法，当 extender 和 extendee 都包含组合关系时，正确交织选择器序列。

#### Scenario: 后代选择器 weave

- **GIVEN** extender = `.a .b`，extendee = `.e`，规则选择器 = `.e .f`
- **WHEN** 系统用 extender 替换 extendee 在原选择器中的位置
- **THEN** 结果 MUST 为 `.a .b .f`

#### Scenario: 子选择器 weave

- **GIVEN** extender = `.a > .b`，extendee = `.e`，规则选择器 = `.e > .f`
- **WHEN** 系统执行 weave
- **THEN** 结果 MUST 保留 extender 的 combinator（`>`)，生成 `.a > .b > .f`

### Requirement: :is() 伪类穿透

系统 SHALL 识别 `:is()`、`:matches()`、`:where()` 等选择器伪类，递归将 extend 应用到伪类参数内部的选择器。

#### Scenario: :is() 内部选择器被扩展

- **GIVEN** `:is(midstream) {@extend upstream}` 和 `downstream {@extend midstream}`，规则 = `upstream {a: b}`
- **WHEN** 系统处理 `downstream @extend midstream`
- **THEN** 系统 MUST 在 `:is(midstream)` 内部找到 `midstream` 并替换为 `:is(midstream, downstream)`
- **AND** 最终结果 MUST 为 `upstream, :is(midstream), :is(midstream, downstream) {a: b}`

### Requirement: serialize 调用结构化 extend

系统 SHALL 在 extend 应用中使用 `selector::extend` 模块的结构化 `SelectorList` 匹配，MUST NOT 使用字符串 `contains` 匹配。

#### Scenario: 不使用字符串匹配

- **WHEN** 系统检查一条 CSS 规则的选择器是否匹配某个 extendee
- **THEN** 系统 MUST 使用 `SelectorList` 和 `ComplexSelector` 类型的结构化匹配
- **AND** 系统 MUST NOT 使用 `str::contains` 做选择器匹配
