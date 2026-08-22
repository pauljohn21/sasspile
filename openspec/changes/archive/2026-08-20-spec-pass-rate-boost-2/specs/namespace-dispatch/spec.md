## ADDED Requirements

### Requirement: 命名空间函数映射全覆盖
系统 SHALL 将所有 sass 内建模块的 `module.function` 形式调用映射到对应内建函数名，不遗漏任何已注册函数。

#### Scenario: map 命名空间函数
- **WHEN** 调用 `map.map-get($map, key)` 或 `map.map-merge($m1, $m2)` 或 `map.map-keys($map)` 或 `map.map-values($map)` 或 `map.map-has-key($map, key)` 或 `map.map-remove($map, key)` 或 `map.map-set($map, key, val)` 或 `map.map-deep-merge($m1, $m2)` 或 `map.map-deep-remove($map, key)`
- **THEN** 系统 正确执行对应 map 内建函数

#### Scenario: string 命名空间函数
- **WHEN** 调用 `string.str-length("hi")` 或 `string.str-index("hi", "i")` 或 `string.str-insert("hi", "x", 1)` 或 `string.str-slice("hi", 1, 1)` 或 `string.str-split("a,b", ",")`
- **THEN** 系统 正确执行对应 string 内建函数

#### Scenario: color 命名空间函数
- **WHEN** 调用 `color.lighten(#800, 20%)` 或 `color.darken(#fff, 20%)` 或 `color.ie-hex-str(#fff)`
- **THEN** 系统 正确执行对应 color 内建函数

#### Scenario: selector 命名空间函数
- **WHEN** 调用 `selector.selector-append(".a", ".b")` 或 `selector.selector-nest(".a", ".b")` 或 `selector.selector-extend(".a", ".a", ".b")` 或 `selector.selector-parse(".a .b")` 或 `selector.selector-replace(".a", ".a", ".b")` 或 `selector.selector-unify(".a", ".b")` 或 `selector.selector-simple-selectors(".a")` 或 `selector.selector-is-superselector(".a", ".a.b")`
- **THEN** 系统 正确执行对应 selector 内建函数

#### Scenario: math 命名空间函数
- **WHEN** 调用 `math.unitless(5px)` 或 `math.comparable(1px, 1em)`
- **THEN** 系统 正确执行对应 math 内建函数

#### Scenario: meta 命名空间函数
- **WHEN** 调用 `meta.load-css("module")` 或 `meta.apply(meta.get-mixin("foo"))` 或 `meta.accepts-content(meta.get-mixin("foo"))`
- **THEN** 系统 正确识别并执行对应 meta 功能
