# Built-in Modules Specification

## Purpose

定义 Sass 内置模块的实现（`sass:color`、`sass:math`、`sass:string`、`sass:list`、`sass:map`、`sass:meta`、`sass:selector`），覆盖 Bootstrap 编译所需的所有内置函数。原生 `SassOp` 系统通过多播事件调用这些函数。

## Requirements

### Requirement: 系统 SHALL 提供 `sass:color` 内置模块
系统 SHALL 实现以下 color 函数：`darken($color, $amount)`、`lighten($color, $amount)`、`mix($color1, $color2, $weight)`、`rgba($color, $alpha)`、`transparentize($color, $amount)`、`opacify($color, $amount)`。

#### Scenario: darken 函数
- **WHEN** 调用 `darken(#fff, 10%)`
- **THEN** 返回颜色变暗 10% 后的颜色值

#### Scenario: lighten 函数
- **WHEN** 调用 `lighten(#000, 100%)`
- **THEN** 返回白色

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

#### Scenario: math.abs 函数
- **WHEN** 调用 `math.abs(-42)`
- **THEN** 返回 `42`

### Requirement: 系统 SHALL 提供 `sass:string` 内置模块
系统 SHALL 实现以下 string 函数：`string.index($string, $substring)`、`string.length($string)`、`string.slice($string, $start, $end)`、`string.to-upper-case($string)`、`string.to-lower-case($string)`。

#### Scenario: string.index 函数
- **WHEN** 调用 `string.index("hello world", "world")`
- **THEN** 返回 `7`

#### Scenario: string.length 函数
- **WHEN** 调用 `string.length("hello")`
- **THEN** 返回 `5`

#### Scenario: string.to-upper-case 函数
- **WHEN** 调用 `string.to-upper-case("hello")`
- **THEN** 返回 `"HELLO"`

### Requirement: 系统 SHALL 提供 `sass:list` 内置模块
系统 SHALL 实现以下 list 函数：`list.append($list, $val)`、`list.index($list, $val)`、`list.length($list)`、`list.nth($list, $n)`、`list.join($list1, $list2)`。

#### Scenario: list.nth 函数
- **WHEN** 调用 `list.nth((a, b, c), 2)`
- **THEN** 返回 `b`

#### Scenario: list.append 函数
- **WHEN** 调用 `list.append((a, b), c)`
- **THEN** 返回 `(a, b, c)`

#### Scenario: list.length 函数
- **WHEN** 调用 `list.length((a, b, c))`
- **THEN** 返回 `3`

### Requirement: 系统 SHALL 提供 `sass:map` 内置模块
系统 SHALL 实现以下 map 函数：`map.get($map, $key)`、`map.has-key($map, $key)`、`map.keys($map)`、`map.merge($map1, $map2)`、`map.remove($map, $keys...)`、`map.values($map)`。

#### Scenario: map.get 函数
- **WHEN** 调用 `map.get(("blue": #0d6efd, "red": #dc3545), "blue")`
- **THEN** 返回 `#0d6efd`

#### Scenario: map.has-key 函数
- **WHEN** 调用 `map.has-key(("a": 1), "a")`
- **THEN** 返回 `true`

#### Scenario: map.merge 函数
- **WHEN** 调用 `map.merge(("a": 1), ("b": 2))`
- **THEN** 返回 `("a": 1, "b": 2)`

### Requirement: 内置函数调用 SHALL 返回正确类型的值
所有内置函数 SHALL 返回 `Value` 类型。数字函数返回 `Value::Number`，字符串函数返回 `Value::String`，列表函数返回 `Value::List`，Map 函数返回 `Value::Map`。类型错误的参数 SHALL 返回错误。

#### Scenario: 类型错误
- **WHEN** 调用 `string.index(123, "world")`（首参数非字符串）
- **THEN** 返回 `Err(crate::Error)` 提示类型不匹配

### Requirement: 系统 SHALL 提供 shade-color 和 tint-color 函数
系统 SHALL 实现 `shade-color($color, $weight)` 和 `tint-color($color, $weight)` 函数。`shade-color` 按权重比例与黑色混合；`tint-color` 按权重比例与白色混合。

#### Scenario: shade-color darkens
- **WHEN** 调用 `shade-color(#0d6efd, 15%)`
- **THEN** 返回 `mix(#000, #0d6efd, 15%)` 等价结果

#### Scenario: tint-color lightens
- **WHEN** 调用 `tint-color(#0d6efd, 15%)`
- **THEN** 返回 `mix(#fff, #0d6efd, 15%)` 等价结果

#### Scenario: weight bounds
- **WHEN** 调用 `shade-color(#fff, 100%)`
- **THEN** 返回 `#000000`
- **WHEN** 调用 `tint-color(#000, 100%)`
- **THEN** 返回 `#ffffff`

### Requirement: 系统 SHALL 提供颜色通道提取函数
系统 SHALL 实现 `red($color)`, `green($color)`, `blue($color)`, `alpha($color)` 四个通道提取函数，返回对应通道的数值。

#### Scenario: red channel
- **WHEN** 调用 `red(#ff0000)`
- **THEN** 返回 `255`

#### Scenario: green channel
- **WHEN** 调用 `green(#00ff00)`
- **THEN** 返回 `255`

#### Scenario: blue channel
- **WHEN** 调用 `blue(#0000ff)`
- **THEN** 返回 `255`

#### Scenario: alpha channel
- **WHEN** 调用 `alpha(rgba(0,0,0,0.5))`
- **THEN** 返回 `0.5`

#### Scenario: hex color channel
- **WHEN** 调用 `red(#0d6efd)`
- **THEN** 返回 `13`
- **WHEN** 调用 `green(#0d6efd)`
- **THEN** 返回 `110`
- **WHEN** 调用 `blue(#0d6efd)`
- **THEN** 返回 `253`

### Requirement: 系统 SHALL 提供 to-rgb 函数
系统 SHALL 实现 `to-rgb($color)` 函数，返回 `R, G, B` comma-separated 格式的字符串，用于 CSS Custom Properties 的 RGB 变量声明。

#### Scenario: to-rgb returns comma-separated
- **WHEN** 调用 `to-rgb(#0d6efd)`
- **THEN** 返回 `"13, 110, 253"`

#### Scenario: to-rgb with interpolation
- **WHEN** 声明 `--bs-primary-rgb: #{to-rgb(#0d6efd)};`
- **THEN** 产物 SHALL 包含 `--bs-primary-rgb: 13, 110, 253;`

### Requirement: 系统 SHALL 提供 str-replace 函数
系统 SHALL 实现 `str-replace($string, $search, $replacement)` 函数，将 `$string` 中所有 `$search` 子串替换为 `$replacement`。

#### Scenario: basic replacement
- **WHEN** 调用 `str-replace("foobar", "foo", "baz")`
- **THEN** 返回 `"bazbar"`

#### Scenario: multiple occurrences
- **WHEN** 调用 `str-replace("aaa", "a", "b")`
- **THEN** 返回 `"bbb"`

#### Scenario: SVG content replacement
- **WHEN** 调用 `str-replace($svg, "%23", "#")`
- **THEN** 返回替换后的字符串

### Requirement: 系统 SHALL 提供 map-keys 和 map-values 函数
系统 SHALL 实现 `map-keys($map)` 和 `map-values($map)` 函数，分别返回 map 所有键的列表和所有值的列表。

#### Scenario: map-keys
- **WHEN** 调用 `map-keys(("a": 1, "b": 2, "c": 3))`
- **THEN** 返回列表 `("a", "b", "c")`

#### Scenario: map-values
- **WHEN** 调用 `map-values(("a": 1, "b": 2, "c": 3))`
- **THEN** 返回列表 `(1, 2, 3)`

#### Scenario: empty map
- **WHEN** 调用 `map-keys(());`
- **THEN** 返回空列表 `()`

### Requirement: 系统 SHALL 提供 map-has-key 函数
系统 SHALL 实现 `map-has-key($map, $key)` 函数，检查 map 是否包含指定键，返回布尔值。

#### Scenario: existing key
- **WHEN** 调用 `map-has-key(("a": 1, "b": 2), "a")`
- **THEN** 返回 `true`

#### Scenario: missing key
- **WHEN** 调用 `map-has-key(("a": 1, "b": 2), "c")`
- **THEN** 返回 `false`

### Requirement: 系统 SHALL 提供 list-separator 函数
系统 SHALL 实现 `list-separator($list)` 函数，返回列表的分隔符类型（`"space"` 或 `"comma"`）。

#### Scenario: space-separated
- **WHEN** 调用 `list-separator(1px solid red)`
- **THEN** 返回 `"space"`

#### Scenario: comma-separated
- **WHEN** 调用 `list-separator((1, 2, 3))`
- **THEN** 返回 `"comma"`

### Requirement: 系统 SHALL 提供 list-zip 函数
系统 SHALL 实现 `list-zip($lists...)` 函数，将多个列表按位置组合为嵌套列表。

#### Scenario: zip two lists
- **WHEN** 调用 `list-zip(a b c, 1 2 3)`
- **THEN** 返回 `((a 1), (b 2), (c 3))`

### Requirement: rgba() 接受 CSS Variable
系统 SHALL 接受 `rgba($color, $alpha)` 第一个参数为 CSS Variable 表达式，返回 `rgba(var(--name-rgb), $alpha)` 格式。

#### Scenario: rgba with CSS var
- **WHEN** 调用 `rgba(var(--bs-white), 0.5))`
- **THEN** 返回 `rgba(var(--bs-white-rgb), 0.5)`
