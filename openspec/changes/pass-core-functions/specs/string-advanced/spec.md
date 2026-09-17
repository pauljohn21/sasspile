# string-advanced Specification

## ADDED Requirements

### Requirement: string.insert

 SHALL 在 `$index` 处插入 `$insert`（1-based，0 视为字符串开头）。

#### Scenario: insert at position 2
- **WHEN** 输入为 `a {b: string.insert("abcd", "X", 2)}`
- **THEN** 输出 `"aXbcd"`

### Requirement: string.slice

 SHALL 从 `$start-at` (1-based) 到 `$end-at` (inclusive, 1-based) 截取子串。负数从末尾算。

#### Scenario: slice middle
- **WHEN** 输入为 `a {b: string.slice("abcd", 2, 3)}`
- **THEN** 输出 `"bc"`

### Requirement: string.split

 SHALL 按 `$separator` 分割字符串返回 list-like 字符串。

#### Scenario: split by dash
- **WHEN** 输入为 `a {b: string.split("a-b-c", "-")}`
- **THEN** 输出形如 `a, b, c` 或 dart-sass 兼容格式

### Requirement: string.to-upper-case / string.to-lower-case

#### Scenario: to-upper-case
- **WHEN** 输入为 `a {b: string.to-upper-case("abc")}`
- **THEN** 输出 `"ABC"`

### Requirement: string.unique-id

 SHALL 返回唯一标识符字符串（格式类似 `u-abc123`）。

#### Scenario: unique-id format
- **WHEN** 调用 `string.unique-id()`
- **THEN** 输出匹配模式 `^u-[0-9a-z]+$`

### Requirement: string.replace

 SHALL 用 `$replacement` 替换所有 `$target` 的出现。

#### Scenario: replace substring
- **WHEN** 输入为 `a {b: string.replace("abcabc", "b", "X")}`
- **THEN** 输出 `"aXcaXc"`
