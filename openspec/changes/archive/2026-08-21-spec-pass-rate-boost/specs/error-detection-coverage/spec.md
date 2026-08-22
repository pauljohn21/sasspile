## ADDED Requirements

### Requirement: 表达式语法错误检测
系统 SHALL 在表达式解析中检测无效语法并报错，而非静默跳过。

#### Scenario: not 后无有效表达式
- **WHEN** 解析 `not` 后不跟有效表达式（如 `not not`）
- **THEN** 系统 报语法错误

#### Scenario: and/or 后无有效表达式
- **WHEN** 解析 `a and` 或 `a or` 后不跟有效表达式
- **THEN** 系统 报语法错误

#### Scenario: 空括号
- **WHEN** 解析 `()`
- **THEN** 系统 报语法错误

#### Scenario: or 前无有效表达式
- **WHEN** 解析 `or b`（or 前无操作数）
- **THEN** 系统 报语法错误

### Requirement: selector 函数错误检测
系统 SHALL 在 selector 函数中检测无效输入并报错。

#### Scenario: append 无效选择器类型
- **WHEN** 调用 `selector-append(123, ".b")`
- **THEN** 系统 报类型错误

#### Scenario: append 无效组合器
- **WHEN** 调用 `selector-append("> .a", ".b")`
- **THEN** 系统 报选择器错误

### Requirement: map 类型检查
系统 SHALL 在 map 函数中验证参数类型，对非 map 输入报错。

#### Scenario: deep_merge 非 map 参数
- **WHEN** 调用 `map-deep-merge(1, (a: 1))`
- **THEN** 系统 报 "$map: 1 is not a map" 错误

#### Scenario: 重复键检测
- **WHEN** map 字面量中存在重复键
- **THEN** 系统 报 "Duplicate key" 错误

### Requirement: @use/@forward conflict 检测
系统 SHALL 在模块加载时检测同名成员 conflict 并报错。

#### Scenario: 变量 conflict
- **WHEN** 两个 `@forward` 导出同名变量
- **THEN** 系统 报 "conflict" 错误

#### Scenario: 函数 conflict
- **WHEN** 两个 `@forward` 导出同名函数
- **THEN** 系统 报 "conflict" 错误

#### Scenario: 同值 conflict
- **WHEN** 两个 `@forward` 导出同名且同值的成员
- **THEN** 系统 不报错（允许同值冲突）

### Requirement: plain CSS 限制
系统 SHALL 在 plain CSS 模式中检测不允许的操作并报错。

#### Scenario: sass() 在 plain CSS
- **WHEN** 在 plain CSS 模式中调用 `sass()`
- **THEN** 系统 报 "sass() conditions aren't allowed in plain CSS" 错误

#### Scenario: 插值在 plain CSS
- **WHEN** 在 plain CSS 模式中使用 `#{...}` 插值在不允许的位置
- **THEN** 系统 报 "Interpolation isn't allowed in plain CSS" 错误
