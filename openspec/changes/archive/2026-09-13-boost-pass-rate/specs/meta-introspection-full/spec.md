## ADDED Requirements

### Requirement: get-function returns callable
`get-function($name)` SHALL return a Function value that can be invoked via `call()`.

#### Scenario: get-function returns callable
- **WHEN** `$fn: get-function("math.div"); call($fn, 10px, 2px)` is evaluated
- **THEN** result is `5px`

#### Scenario: get-function with namespace
- **WHEN** `$fn: get-function("math.div")` inside a `@use "math"` block
- **THEN** returns the namespaced function, call() succeeds

### Requirement: keywords returns map
`keywords($args)` SHALL return a Map with named keys rather than a list.

#### Scenario: keywords map-get
- **WHEN** `@mixin foo($args...) { @debug map-get(keywords($args), bar); } @include foo($bar: 1px)` is evaluated
- **THEN** result is `1px`

### Requirement: global_variable_exists checks global scope
`global_variable_exists($name)` SHALL return true regardless of current local scope.

#### Scenario: Variable exists in global not local
- **WHEN** `$global: 1; @function test() { @return global_variable_exists("global"); }`
- **THEN** `test()` returns `true`

#### Scenario: dash-insensitive name lookup
- **WHEN** `$my-var: 1; global_variable_exists("my_var")` is evaluated
- **THEN** returns `true` (dash and underscore are equivalent)

### Requirement: function-exists builtin-function
`function-exists($name)` SHALL return true for built-in functions like `math.div`.

#### Scenario: Check built-in
- **WHEN** `function-exists("math.div")` is evaluated
- **THEN** returns `true`

#### Scenario: 2-arg form with namespace
- **WHEN** `function-exists("div", "math")` is evaluated inside non-math scope
- **THEN** returns `true`
