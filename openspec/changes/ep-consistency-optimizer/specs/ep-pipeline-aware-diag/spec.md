## ADDED Requirements

### Requirement: ep_normalized_test 在比较前剥离 autoprefixer 注入属性
测试管线 SHALL 在 lightningcss minify 之后、比较之前，移除由 autoprefixer 注入的 CSS 属性（这些属性不在 sasspile 控制范围内）。

#### Scenario: -webkit-user-select 差异不影响比较
- **WHEN** EP dist 含 `-webkit-user-select: none` 而 sasspile 输出不含
- **THEN** 该属性 SHALL 被从比较中排除，不标记为 DIFF

#### Scenario: -webkit-appearance 差异不影响比较
- **WHEN** EP dist 含 `-webkit-appearance: none` 而 sasspile 输出仅含 `appearance: none`
- **THEN** `-webkit-appearance` SHALL 被从比较中排除

#### Scenario: -moz-user-select / -ms-user-select 差异不影响比较
- **WHEN** EP dist 含 `-moz-user-select` 或 `-ms-user-select` 而 sasspile 输出不含
- **THEN** 这些属性 SHALL 被排除

#### Scenario: 真正语义差异仍被检测
- **WHEN** sasspile 输出缺少 EP dist 中的 `user-select: none`（非前缀版）
- **THEN** 该差异 SHALL 被标记为真正 DIFF（sasspile bug）

### Requirement: 对比管线支持逐属性差异分类
对于 lightningcss normalize 后的 CSS，对比逻辑 SHALL 输出逐属性级别的差异分类（autoprefixer / semantic / selector-formatting），便于快速定位根因。

#### Scenario: 逐属性 diff 输出
- **WHEN** sasspile 和 EP dist 在同一选择器下某属性值不同
- **THEN** 输出 SHALL 标明选择器、属性名、sasspile 值、EP dist 值
