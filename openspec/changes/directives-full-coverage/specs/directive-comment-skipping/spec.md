## ADDED Requirements

### Requirement: Parser SHALL skip comments between directive keyword and its arguments
When parsing `@for`, `@mixin`, `@if`, `@function`, `@while`, `@warn`, `@error`, `@include`, `@use`, `@forward` directives, the parser SHALL skip any `/* */` or `//` comments that appear between the directive keyword and the first meaningful token.

#### Scenario: Block comment after @for keyword
- **WHEN** the input is `@for /**/ $i from 1 through 10 {}`
- **THEN** the parser SHALL produce the same AST as `@for $i from 1 through 10 {}`

#### Scenario: Block comment after @for variable before `from`
- **WHEN** the input is `@for $i /**/ from 1 through 10 {}`
- **THEN** the parser SHALL produce a valid for-loop AST

#### Scenario: Block comment before `through` keyword
- **WHEN** the input is `@for $i from 1 /**/ through 10 {}`
- **THEN** the parser SHALL produce a valid for-loop AST with through keyword

#### Scenario: Block comment after @if keyword
- **WHEN** the input is `@if /**/ condition {}`
- **THEN** the parser SHALL produce the same AST as `@if condition {}`

#### Scenario: Block comment after @mixin keyword
- **WHEN** the input is `@mixin /**/ name() {}`
- **THEN** the parser SHALL produce a valid mixin definition AST

#### Scenario: Block comment after @function keyword
- **WHEN** the input is `@function /**/ name() {@return 1}`
- **THEN** the parser SHALL produce a valid function definition AST

#### Scenario: Silent comment after @for keyword
- **WHEN** the input is `@for // comment\n $i from 1 through 10 {}`
- **THEN** the parser SHALL skip the silent comment and produce a valid for-loop AST

#### Scenario: Block comment after @include keyword before mixin name
- **WHEN** the input is `@include /**/ mixin-name;`
- **THEN** the parser SHALL produce a valid include AST

#### Scenario: Block comment in @function arguments
- **WHEN** the input is `@function name(/**/ $arg) {@return $arg}`
- **THEN** the parser SHALL produce a valid function definition with parameter `$arg`

#### Scenario: Block comment after @while keyword
- **WHEN** the input is `@while /**/ $i > 0 {}`
- **THEN** the parser SHALL produce a valid while-loop AST
