## ADDED Requirements

### Requirement: Expression Evaluator SHALL support arithmetic operators
The evaluator SHALL compute `+`, `-`, `*`, `/`, `%` with proper unit handling and type coercion.

#### Scenario: Number arithmetic
- **WHEN** evaluating `10px + 5px`
- **THEN** result is `Number(15, Some(Px))`

#### Scenario: Division with parentheses
- **WHEN** evaluating `(100px / 2)`
- **THEN** result is `Number(50, Some(Px))` — parens required for division operator

#### Scenario: Unit mismatch error
- **WHEN** evaluating `10px + 5em`
- **THEN** evaluator produces error "Incompatible units: px and em"

### Requirement: Expression Evaluator SHALL support string operations
The evaluator SHALL handle string concatenation via `+` and interpolation `#{}`.

#### Scenario: String concatenation
- **WHEN** evaluating `"hello " + "world"`
- **THEN** result is `String("hello world")` (unquoted)

#### Scenario: Interpolation evaluation
- **WHEN** evaluating `"item-#{$i}"` with `$i: 3`
- **THEN** result is `String("item-3")`

### Requirement: Expression Evaluator SHALL support comparison operators
The evaluator SHALL compute `==`, `!=`, `<`, `>`, `<=`, `>=` returning boolean values.

#### Scenario: Numeric comparison
- **WHEN** evaluating `10 > 5`
- **THEN** result is `Boolean(true)`

#### Scenario: String equality
- **WHEN** evaluating `"a" == "a"`
- **THEN** result is `Boolean(true)`

### Requirement: Expression Evaluator SHALL support logical operators
The evaluator SHALL handle `and`, `or`, `not` with short-circuit semantics where applicable.

#### Scenario: and short-circuit
- **WHEN** evaluating `false and $undefined`
- **THEN** result is `Boolean(false)` without evaluating `$undefined`

#### Scenario: not operator
- **WHEN** evaluating `not (true)`
- **THEN** result is `Boolean(false)`

### Requirement: Expression Evaluator SHALL resolve function calls
The evaluator SHALL invoke user-defined functions and builtins, passing evaluated arguments.

#### Scenario: User function call
- **WHEN** `@function double($n) { @return $n * 2; }` then calling `double(21)`
- **THEN** result is `Number(42, None)`

#### Scenario: Builtin color function
- **WHEN** evaluating `red(#cc0000)`
- **THEN** result is `Number(204, None)`

### Requirement: Expression Evaluator SHALL support list and map access
The evaluator SHALL handle `list.nth()`, `$map[key]`, and `$map.key` access patterns.

#### Scenario: nth access
- **WHEN** evaluating `nth(10px 20px 30px, 2)`
- **THEN** result is `Number(20, Some(Px))`

#### Scenario: Map key access
- **WHEN** evaluating `("a": 1, "b": 2).a`
- **THEN** result is `Number(1, None)`

### Requirement: Expression Evaluator SHALL support if() ternary function
The evaluator SHALL handle the builtin `if($condition, $if-true, $if-false)` function.

#### Scenario: True branch
- **WHEN** evaluating `if(true, yes, no)`
- **THEN** result is `String("yes")`

#### Scenario: False branch
- **WHEN** evaluating `if(false, yes, no)`
- **THEN** result is `String("no")`
