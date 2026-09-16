## ADDED Requirements

### Requirement: nth 1-based 索引
`nth($list, $n)` MUST 使用 1-based 索引，返回 list 中第 $n 个元素。

#### Scenario: nth 基本
- **WHEN** SCSS: `a { b: nth(red green blue, 2); }`
- **THEN** MUST 输出 `green`

### Requirement: length 统计元素
`length($list)` MUST 返回 list 中元素个数（空格/逗号分隔）。

#### Scenario: length 多分隔符
- **WHEN** SCSS: `a { b: length(red green blue); }`
- **THEN** MUST 输出 `3`

### Requirement: append 追加元素
`append($list, $val)` MUST 返回追加 $val 的新列表。

#### Scenario: append 基本
- **WHEN** SCSS: `a { b: append(red blue, green); }`
- **THEN** MUST 输出包含 `green` 的列表形式

### Requirement: map-get 命中
`map-get($map, $key)` MUST 按键返回对应值。

#### Scenario: map-get 命中
- **WHEN** SCSS: `$config: (color: red, size: 10px); a { b: map-get($config, color); }`
- **THEN** MUST 输出 `red`

### Requirement: map-has-key 验证键存在
`map-has-key($map, $key)` MUST 返回 "true" 或 "false" 字符串。

#### Scenario: map-has-key true/false
- **WHEN** SCSS: `$config: (color: red); a { b: map-has-key($config, color); } c { b: map-has-key($config, missing); }`
- **THEN** `a` 输出 `true`，`c` 输出 `false`

### Requirement: quote 添加引号
`quote($string)` MUST 返回带双引号的字符串。

#### Scenario: quote 基本
- **WHEN** SCSS: `a { b: quote(hello); }`
- **THEN** MUST 输出 `"hello"`

### Requirement: unquote 移除引号
`unquote($string)` MUST 移除字符串两端引号。

#### Scenario: unquote 基本
- **WHEN** SCSS: `a { b: unquote("hello"); }`
- **THEN** MUST 输出 `hello`
