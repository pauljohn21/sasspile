## ADDED Requirements

### Requirement: math 函数命名参数支持
系统 SHALL 在所有 math 内建函数中支持命名参数传递，合并位置参数和命名参数后进行验证。

#### Scenario: atan2 命名参数
- **WHEN** 调用 `math.atan2($y: 1, $x: 2)`
- **THEN** 系统 正确计算 atan2(1, 2) 的值

#### Scenario: sin 命名参数
- **WHEN** 调用 `math.sin($number: 0)`
- **THEN** 系统 返回 0

#### Scenario: pow 命名参数
- **WHEN** 调用 `math.pow($base: 2, $exponent: 3)`
- **THEN** 系统 返回 8

### Requirement: CSS 函数多参数透传
系统 SHALL 在 CSS 上下文中将 `clamp`/`min`/`max` 等函数的多参数调用透传为 CSS 原生函数输出，而非报参数错误。

#### Scenario: clamp CSS 透传
- **WHEN** 在 CSS 声明值中调用 `clamp(1%, 2vw, 3%)`
- **THEN** 系统 输出 `clamp(1%, 2vw, 3%)` 而非报 "Only 1 argument allowed"

#### Scenario: min CSS 透传
- **WHEN** 在 CSS 声明值中调用 `min(100px, 50vw)`
- **THEN** 系统 输出 `min(100px, 50vw)` 而非报参数错误

### Requirement: selector 函数参数展开
系统 SHALL 在 selector 函数中正确处理多参数展开和命名参数。

#### Scenario: selector-parse 参数
- **WHEN** 调用 `selector-parse(".a .b")`
- **THEN** 系统 返回 `(".a" ".b")` 形式的选择器列表

#### Scenario: selector-extend 命名参数
- **WHEN** 调用 `selector-extend(".a", ".a", ".b")`
- **THEN** 系统 返回扩展后的选择器

### Requirement: string 函数参数验证
系统 SHALL 在 string 函数中正确验证参数类型，对非字符串参数报类型错误而非参数数量错误。

#### Scenario: str-length 参数
- **WHEN** 调用 `str-length("hello")`
- **THEN** 系统 返回 5

#### Scenario: str-index 参数类型
- **WHEN** 调用 `str-index("hello", "ll")`
- **THEN** 系统 返回 3
