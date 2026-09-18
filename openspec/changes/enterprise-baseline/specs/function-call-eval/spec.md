# function-call-eval Specification

## ADDED Requirements

### Requirement: @function user-defined function and @return

`@function add($a, $b) { @return $a + $b; }` SHALL register; `add(1px, 2px)` SHALL evaluate as a pure function inside the reducer returning `3px`.

#### Scenario: @function 输出用于声明值
- **WHEN** `@function double($x) { @return $x * 2; } a { w: double(5px); }`
- **THEN** 输出 `a{w:10px}`

### Requirement: map-get / map-keys built-in functions

`map-get((a: 1, b: 2), a)` SHALL return `1`. `map-keys(...)` SHALL return the key list.

#### Scenario: map-get 嵌套使用
- **WHEN** `$m: (gap: 8px); a { p: map-get($m, gap); }`
- **THEN** 输出 `a{p:8px}`
