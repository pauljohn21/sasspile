## ADDED Requirements

### Requirement: @extend 跨文件选择器传递

系统 SHALL 正确处理 @extend 在跨文件（@use/@import）场景下的选择器传递。

#### Scenario: bogus 选择器输出
- **WHEN** `@extend .bogus` 被执行且 `.bogus` 选择器不存在于任何文件中
- **THEN** 系统 SHALL 输出包含 bogus 选择器的占位规则（匹配 sass-spec 的 trailing/leading/middle 模式）

#### Scenario: pseudo 嵌套 extend
- **WHEN** `@extend :hover` 被执行且目标选择器包含 pseudo 伪类
- **THEN** 系统 SHALL 正确生成嵌套 pseudo 选择器（如 `a:hover, .extender:hover`）

#### Scenario: diamond 依赖
- **WHEN** 模块 A @use 模块 B 和模块 C，模块 B 和 C 都 @extend 同一选择器
- **THEN** 系统 SHALL 正确合并 diamond 依赖的 @extend 关系

#### Scenario: midstream extend in pseudoselector
- **WHEN** @extend 出现在 pseudo 选择器中间位置（如 `a:hover` 内部）
- **THEN** 系统 SHALL 正确处理 three_files 和 two_files 的 is/matches 场景

#### Scenario: optional and mandatory
- **WHEN** @extend 带 optional 标记和不带 optional 标记混合使用
- **THEN** 系统 SHALL 区分 optional（找不到目标时静默）和 mandatory（找不到目标时报错）行为
