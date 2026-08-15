## ADDED Requirements

### Requirement: Parser SHALL produce AST from Token stream
The parser SHALL consume tokens produced by the lexer and build a structured AST representing the SCSS document.

#### Scenario: Rule with declarations
- **WHEN** parser receives tokens for `a { color: red; }`
- **THEN** it produces `Node::Rule` with selector `a` and declaration `color: red`

#### Scenario: Nested selectors
- **WHEN** parser processes nested SCSS selectors
- **THEN** it produces correctly nested `Rule` nodes matching sass-spec nesting behavior

#### Scenario: At-rules
- **WHEN** parser encounters `@media`, `@supports`, `@mixin`, `@include`
- **THEN** it produces corresponding `AtRule` variants in the AST

### Requirement: Parser SHALL handle all Sass directives
The parser SHALL recognize and parse: `@use`, `@import`, `@forward`, `@mixin`, `@include`, `@function`, `@return`, `@if`, `@else`, `@for`, `@each`, `@while`, `@extend`, `@at-root`, `@media`, `@supports`, `@content`, `@debug`, `@warn`, `@error`.

#### Scenario: @use with configuration
- **WHEN** parser encounters `@use "module" as m with ($var: default)`
- **THEN** it produces `AtRule::Use` with namespace `m` and configuration map

#### Scenario: @mixin with parameters
- **WHEN** parser encounters `@mixin foo($a, $b: default) { ... }`
- **THEN** it produces `AtRule::Mixin` with parameter list including optional defaults

### Requirement: Parser SHALL handle interpolation
The parser SHALL parse `#{...}` interpolation in selectors, property names, values, and strings.

#### Scenario: Selector interpolation
- **WHEN** parser processes `.#{$class}-suffix`
- **THEN** it produces selector with `Interpolation` expression node

### Requirement: Parser SHALL produce detailed error messages
When encountering invalid syntax, the parser SHALL produce actionable error messages with source location.

#### Scenario: Unmatched brace
- **WHEN** parser encounters `a { color: red` without closing brace
- **THEN** it produces error "expected '}' at line N, found end of file"
