## ADDED Requirements

### Requirement: @import 文件冲突检测

系统 SHALL 在 `@import` 加载文件时检测文件冲突，包括 partial/extension/all/index/import_only 场景。

#### Scenario: partial 冲突
- **WHEN** 同一目录下存在 `_foo.scss` 和 `foo.scss`，且 `@import "foo"` 被执行
- **THEN** 系统 SHALL 报错，错误消息说明文件冲突

#### Scenario: extension 冲突
- **WHEN** 同一目录下存在 `foo.scss` 和 `foo.css`，且 `@import "foo"` 被执行
- **THEN** 系统 SHALL 报错，错误消息说明扩展名冲突

#### Scenario: all 冲突
- **WHEN** 同一目录下存在多个匹配文件（如 `_foo.scss`、`foo.scss`、`foo.sass`）
- **THEN** 系统 SHALL 报错，错误消息列出所有匹配文件

#### Scenario: index 冲突
- **WHEN** 同一目录下存在 `_index.scss` 和 `index.scss`
- **THEN** 系统 SHALL 报错，错误消息说明 index 冲突

#### Scenario: import_only 冲突
- **WHEN** 目录下只有 `.import.scss` 文件和普通 `.scss` 文件
- **THEN** 系统 SHALL 根据 sass-spec 规则选择正确的文件或报错

### Requirement: 顶层声明错误检测

系统 SHALL 检测在顶层上下文中不允许的声明。

#### Scenario: 顶层 @include
- **WHEN** `@include` 出现在文件顶层（非规则体内）且引用的 mixin 产生 CSS 输出
- **THEN** 系统 SHALL 根据 sass-spec 规则处理或报错

#### Scenario: 顶层 @at-root
- **WHEN** `@at-root` 出现在文件顶层（无父选择器）
- **THEN** 系统 SHALL 根据 sass-spec 规则处理或报错
