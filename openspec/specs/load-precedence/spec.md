## ADDED Requirements

### Requirement: @import 文件加载优先级

系统 SHALL 按 sass-spec 规定的优先级顺序解析 @import 文件路径。

#### Scenario: sass 优先于 css
- **WHEN** 同一目录下存在 `foo.scss` 和 `foo.css`，且 `@import "foo"` 被执行
- **THEN** 系统 SHALL 优先加载 `foo.scss`

#### Scenario: partial 优先于普通文件
- **WHEN** 同一目录下存在 `_foo.scss` 和 `foo.scss`
- **THEN** 系统 SHALL 根据 sass-spec 规则选择正确文件或报冲突

#### Scenario: index 优先级
- **WHEN** 目录下存在 `_index.scss` 和 `index.scss`
- **THEN** 系统 SHALL 根据 sass-spec 规则选择正确文件或报冲突

#### Scenario: import_only 隐式扩展
- **WHEN** 目录下只有 `foo.import.scss` 文件
- **THEN** 系统 SHALL 加载 `foo.import.scss`

#### Scenario: import_only 显式扩展
- **WHEN** 目录下有 `foo.scss` 和 `foo.import.scss`
- **THEN** 系统 SHALL 根据 sass-spec 规则选择正确文件或报冲突

#### Scenario: partial 先于 normal
- **WHEN** 目录下有 `_foo.scss` 和 `foo.scss`，import_only 模式
- **THEN** 系统 SHALL 根据 sass-spec 规则处理优先级

#### Scenario: normal 先于 partial
- **WHEN** 目录下有 `foo.scss` 和 `_foo.scss`，import_only 模式
- **THEN** 系统 SHALL 根据 sass-spec 规则处理优先级
