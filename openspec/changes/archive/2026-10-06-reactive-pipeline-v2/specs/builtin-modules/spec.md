# Spec Delta

## Purpose

定义 Sass 内置模块的实现（`sass:color`、`sass:math`、`sass:string`、`sass:list`、`sass:map`、`sass:meta`、`sass:selector`），覆盖 Bootstrap 编译所需的所有内置函数。原生 `SassOp` 系统通过多播事件调用这些函数。

## ADDED Requirements

### Requirement: 系统 SHALL 提供 `sass:color` 内置模块
系统 SHALL 实现以下 color 函数：`darken($color, $amount)`、`lighten($color, $amount)`、`mix($color1, $color2, $weight)`、`rgba($color, $alpha)`、`transparentize($color, $amount)`、`opacify($color, $amount)`、`to-upper-case($string)`。

#### Scenario: darken 函数
- **WHEN** 调用 `darken(#fff, 10%)`
- **THEN** 返回颜色变暗 10% 后的颜色值

#### Scenario: mix 函数
- **WHEN** 调用 `mix(#f00, #00f, 50%)`
- **THEN** 返回红蓝各 50% 混合后的颜色

### Requirement: 系统 SHALL 提供 `sass:math` 内置模块
系统 SHALL 实现以下 math 函数：`math.clamp($min, $val, $max)`、`math.max($numbers...)`、`math.min($numbers...)`、`math.round($number)`、`math.abs($number)`、`math.percentage($number)`。

#### Scenario: percentage 函数
- **WHEN** 调用 `math.percentage(0.5)`
- **THEN** 返回 `50%`

#### Scenario: clamp 函数
- **WHEN** 调用 `math.clamp(0, 1.5, 1)`
- **THEN** 返回 `1`

### Requirement: 系统 SHALL 提供 `sass:string` 内置模块
系统 SHALL 实现以下 string 函数：`string.index($string, $substring)`、`string.length($string)`、`string.slice($string, $start, $end)`、`string.to-upper-case($string)`、`string.to-lower-case($string)`、`string.unique-id()`。

#### Scenario: string.index 函数
- **WHEN** 调用 `string.index("hello world", "world")`
- **THEN** 返回 `7`

### Requirement: 系统 SHALL 提供 `sass:list` 内置模块
系统 SHALL 实现以下 list 函数：`list.append($list, $val)`、`list.index($list, $val)`、`list.length($list)`、`list.nth($list, $n)`、`list.join($list1, $list2)`、`list.separator($list)`、`list.set-nth($list, $n, $val)`、`list.zip($lists...)`。

#### Scenario: list.nth 函数
- **WHEN** 调用 `list.nth((a, b, c), 2)`
- **THEN** 返回 `b`

#### Scenario: list.append 函数
- **WHEN** 调用 `list.append((a, b), c)`
- **THEN** 返回 `(a, b, c)`

### Requirement: 系统 SHALL 提供 `sass:map` 内置模块
系统 SHALL 实现以下 map 函数：`map.get($map, $key)`、`map.has-key($map, $key)`、`map.keys($map)`、`map.merge($map1, $map2)`、`map.remove($map, $keys...)`、`map.values($map)`。

#### Scenario: map.get 函数
- **WHEN** 调用 `map.get(("blue": #0d6efd, "red": #dc3545), "blue")`
- **THEN** 返回 `#0d6efd`

### Requirement: 内置函数调用 SHALL 返回正确类型的值
所有内置函数 SHALL 返回 `Value` 类型。数字函数返回 `Value::Number`，字符串函数返回 `Value::String`，列表函数返回 `Value::List`，颜色函数返回 `Value::Color` 或 `Value::String`。类型错误的参数 SHALL 返回错误。

#### Scenario: 类型错误
- **WHEN** 调用 `string.index(123, "world")`（首参数非字符串）
- **THEN** 返回 `Err(crate::Error)` 提示类型不匹配
