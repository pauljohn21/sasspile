# Spec: Nested SCSS Function Evaluation

## ADDED Requirements

### Requirement: Builtin Functions Accept Nested Function Results

When a builtin function is called with arguments that are themselves function calls, the evaluator SHALL evaluate inner calls first.

#### Scenario: `string.unquote(map.get(...))`

- **WHEN** SCSS contains `#{$selector + string.unquote(map.get($map, $key))}`
- **THEN** `map.get($map, $key)` SHALL be evaluated to get the map value
- **AND** `string.unquote(...)` SHALL be called on the result
- **AND** the final string SHALL be concatenated into the selector

#### Scenario: `getCssVar` in calc expression

- **WHEN** SCSS contains `calc(getCssVar("input-otp-size") - 4px)`
- **THEN** `getCssVar("input-otp-size")` SHALL be evaluated to `var(--el-input-otp-size)`
- **AND** the full expression SHALL be `calc(var(--el-input-otp-size) - 4px)`

### Requirement: User @function Visible in Expression Context

#### Scenario: EP `@function getCssVar` recognized in property values

- **WHEN** SCSS file defines `@function getCssVar(...)` via `@mixin function`
- **AND** that function is called inside a property value interpolation
- **THEN** the call SHALL be dispatched to the user-defined function implementation
- **AND** the return value SHALL be used in the property

#### Scenario: EP `@function getCssVar` recognized in selector interpolation

- **WHEN** SCSS contains `#{$namespace}-breadcrumb__inner`
- **THEN** the variable interpolation SHALL produce the resolved string
- **AND** no unresolved `$variable` text SHALL appear in output
