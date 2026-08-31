## ADDED Requirements

### Requirement: Bare interpolation in CSS declaration values
The system SHALL evaluate `#{$var}` interpolation in CSS declaration values (e.g., `content: #{$a};`) to the variable's actual value, not the literal `$var` text.

#### Scenario: Simple variable interpolation
- **WHEN** SCSS source contains `$a: hello; .b { content: #{$a}; }`
- **THEN** compiled CSS SHALL contain `content: hello;`

#### Scenario: Distributed variables through @forward chain
- **WHEN** `@use 'module' with ($a: 'a')` propagates through `@forward` + `@use as *` to a module where `$a: default !default` is configured
- **THEN** `#{$a}` in the leaf module SHALL evaluate to `a`

#### Scenario: String-quoted interpolation still works
- **WHEN** SCSS source contains `$a: hello; .b { content: "#{$a}"; }`
- **THEN** compiled CSS SHALL contain `content: "hello";`

#### Scenario: Interpolation with prefix and suffix
- **WHEN** SCSS source contains `$a: red; .b { color: prefix-#{$a}-suffix; }`
- **THEN** compiled CSS SHALL contain `color: prefix-red-suffix;`

#### Scenario: Interpolation with expression
- **WHEN** SCSS source contains `.b { width: #{1 + 2}px; }`
- **THEN** compiled CSS SHALL contain `width: 3px;`
