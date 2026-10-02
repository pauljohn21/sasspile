## ADDED Requirements

### Requirement: @keyframes 内部声明顺序
系统 SHALL 对 @keyframes 内部的 @include 输出保持 EP dist 中的声明顺序。

#### Scenario: Mixin 展开在百分比块内
- **WHEN** `@include some-mixin` 在 `@keyframes` 的 `from` 或 `to` 块内
- **THEN** mixin 展开的声明嵌套在正确的百分比节点内

#### Scenario: 空关键帧块
- **WHEN** `@keyframes name { 100% {} }`
- **THEN** 输出 `100% {}` 空块或省略（与 EP dist 一致）
