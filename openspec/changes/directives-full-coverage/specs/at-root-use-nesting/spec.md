## ADDED Requirements

### Requirement: @at-root SHALL allow nested @use rules
The system SHALL permit `@use` rules inside `@at-root` blocks.

#### Scenario: @use inside @at-root with built-in module
- **WHEN** the input is `@at-root { @use "sass:math"; a { b: math.$PI; } }`
- **THEN** the system SHALL compile without error and output the @at-root resolved CSS

#### Scenario: @use inside @at-root with user module
- **WHEN** the input is `@at-root { @use "lib"; a { b: lib.value; } }`
- **THEN** the system SHALL compile without "This at-rule is not allowed here" error

#### Scenario: Multiple @use inside @at-root
- **WHEN** the input is `@at-root { @use "a"; @use "b"; c { d: a.val b.val } }`
- **THEN** the system SHALL compile without error
