## ADDED Requirements

### Requirement: User-defined function names SHALL be case-insensitive
When resolving a function call, the system SHALL treat user-defined function names as case-insensitive. `ELEMENT()`, `Element()`, and `element()` SHALL all refer to the same `@function ELEMENT()` definition.

#### Scenario: Uppercase function definition called with lowercase
- **WHEN** the input defines `@function ELEMENT() {@return 1}` and calls `a { b: element(); }`
- **THEN** the output SHALL be `a { b: 1; }`

#### Scenario: Uppercase function definition called with mixed case
- **WHEN** the input defines `@function ELEMENT() {@return 1}` and calls `a { b: Element(); }`
- **THEN** the output SHALL be `a { b: 1; }`

#### Scenario: Lowercase function definition called with uppercase
- **WHEN** the input defines `@function element() {@return 1}` and calls `a { b: ELEMENT(); }`
- **THEN** the output SHALL be `a { b: 1; }`

#### Scenario: Function name `expression` case-insensitive
- **WHEN** the input defines `@function EXPRESSION() {@return 1}` and calls `a { b: expression(); }`
- **THEN** the output SHALL be `a { b: 1; }`

#### Scenario: Function name `url` case-insensitive
- **WHEN** the input defines `@function URL() {@return 1}` and calls `a { b: url(); }`
- **THEN** the output SHALL be `a { b: 1; }`

#### Scenario: Prefixed function name case-insensitive
- **WHEN** the input defines @function `LIB-element()` {@return 1} in namespace `lib` and calls `a { b: lib.element(); }`
- **THEN** the output SHALL be `a { b: 1; }`

#### Scenario: CSS native function NOT overridden by user function with different case
- **WHEN** no user function `element()` is defined and the input is `a { b: ELEMENT(); }`
- **THEN** the output SHALL preserve `ELEMENT()` as-is (it is a CSS native function)
