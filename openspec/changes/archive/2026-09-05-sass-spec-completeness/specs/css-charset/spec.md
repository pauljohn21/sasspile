## ADDED Requirements

### Requirement: @charset 声明
MUST 解析并输出 `@charset "UTF-8";` 声明，位于文件最顶部。

#### Scenario: charset 声明
- **WHEN** 输入包含 `@charset "UTF-8";`
- **THEN** 输出文件顶部保留 charset 声明

#### Scenario: charset 位置错误
- **WHEN** @charset 不在文件首行
- **THEN** 发出警告或忽略（符合规范行为）
