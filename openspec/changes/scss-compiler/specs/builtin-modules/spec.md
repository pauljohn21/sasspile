## ADDED Requirements

### Requirement: Built-in `@sass:color` module SHALL be implemented
The compiler SHALL expose Sass's built-in color functions: `red()`, `green()`, `blue()`, `alpha()`, `mix()`, `invert()`, `lighten()`, `darken()`, `saturate()`, `desaturate()`, `hue()`, `saturation()`, `lightness()`, `scale()`, `change()`, `adjust()`, `opacify()`, `transparentize()`, `grayscale()`, `complement()`.

#### Scenario: lighten function
- **WHEN** calling `color.lighten(red, 20%)`
- **THEN** result has hue preserved but lightness increased 20 percentage points

#### Scenario: alpha function
- **WHEN** calling `color.alpha(rgba(0,0,0,0.5))`
- **THEN** result is `Number(0.5, None)`

#### Scenario: scale function
- **WHEN** calling `color.scale(red, $lightness: 20%)`
- **THEN` uses scaling algorithm that interpolates toward white/black

### Requirement: Built-in `@sass:math` module SHALL be implemented
The compiler SHALL expose: `math.div()`, `math.round()`, `math.ceil()`, `math.floor()`, `math.abs()`, `math.min()`, `math.max()`, `math.random()`, `math.percentage()`, `math.clamp()`, `math.sin()`, `math.cos()`, `math.tan()`, `math.asin()`, `math.acos()`, `math.atan()`, `math.atan2()`, `math.pow()`, `math.sqrt()`, `math.log()`, `math.hypot()`, `math.$pi`, `math.$e`.

#### Scenario: math.div
- **WHEN** calling `math.div(10px, 2)`
- **THEN** result is `Number(5, Some(Px))` (division as function)

#### Scenario: math.percentage
- **WHEN** calling `math.percentage(0.5)`
- **THEN** result is `Number(50, Some(Percent))`

#### Scenario: math.sin in radians
- **WHEN** calling `math.sin(calc(pi / 2))`
- **THEN** result is approximately `Number(1, None)`

### Requirement: Built-in `@sass:list` module SHALL be implemented
The compiler SHALL expose: `list.join()`, `list.append()`, `list.length()`, `list.nth()`, `list.set-nth()`, `list.index()`, `list.zip()`, `list.separator()`, `list.is-bracketed()`, `list.slash()`.

#### Scenario: list.join
- **WHEN** calling `list.join(10px 20px, 30px)`
- **THEN** result is single list `10px 20px 30px`

#### Scenario: list.append with separator
- **WHEN** calling `list.append(10px 20px, 30px, $separator: comma)`
- **THEN** result is `10px, 20px, 30px`

### Requirement: Built-in `@sass:map` module SHALL be implemented
The compiler SHALL expose: `map.get()`, `map.set()`, `map.merge()`, `map.keys()`, `map.values()`, `map.has-key()`, `map.remove()`, `map.deep-merge()`, `map.deep-get()`, `map.deep-set()`, `map.deep-has-key()`, `map.keys()`, `map.values()`.

#### Scenario: map.get
- **WHEN** calling `map.get(("a": 1, "b": 2), "a")`
- **THEN** result is `Number(1, None)`

#### Scenario: map.deep-merge
- **WHEN** calling `map.deep-merge(("a": ("b": 1)), ("a": ("c": 2)))`
- **THEN** result is single map `("a": ("b": 1, "c": 2))`

### Requirement: Built-in `@sass:string` module SHALL be implemented
The compiler SHALL expose: `string.unquote()`, `string.quote()`, `string.length()`, `string.insert()`, `string.slice()`, `string.index()`, `string.split()`, `string.to-upper-case()`, `string.to-lower-case()`, `string.unique-id()`.

#### Scenario: string.split
- **WHEN** calling `string.split("hello world", " ")`
- **THEN** result is list `("hello", "world")`

#### Scenario: string.unique-id
- **WHEN** calling `string.unique-id()`
- **THEN** result is unquoted string like `"uabc12345"` (generated)

### Requirement: Built-in `@sass:meta` module SHALL be implemented
The compiler SHALL expose: `meta.call()`, `meta.get-function()`, `meta.type-of()`, `meta.content-exists()`, `meta.assert-equals()`, `meta.keywords()`, `meta.inspect()`, `meta.module-functions()`, `meta.module-variables()`, `meta.module-mixins()`, `meta.global-variable-exists()`, `meta.function-exists()`, `meta.mixin-exists()`, `meta.feature-exists()`.

#### Scenario: meta.type-of
- **WHEN** calling `meta.type-of(100px)`
- **THEN** result is `String("number")`

#### Scenario: meta.call
- **WHEN** calling `meta.call(meta.get-function("foo"))`
- **THEN** invokes function `foo` with no arguments
