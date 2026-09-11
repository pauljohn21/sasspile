## ADDED Requirements

### Requirement: @use SHALL be allowed in files @import'd from within CSS rules
When `@import` is nested inside a CSS rule, the imported file SHALL be evaluated in a fresh top-level context where `@use` rules are permitted.

#### Scenario: @import with @use'd builtin module
- **WHEN** `a { @import "other"; }` and `other.scss` contains `@use "sass:math"`
- **THEN** the import SHALL succeed without error, and `@at-root` in other.scss SHALL work

#### Scenario: @import with @use'd user module
- **WHEN** `a { @import "other"; }` and `other.scss` contains `@use "used"` (a user module)
- **THEN** the import SHALL succeed without error

#### Scenario: @import preserving @at-root
- **WHEN** `a { @import "other"; }` and `other.scss` contains `@at-root { b { c: d; } }`
- **THEN** the output SHALL be `b { c: d; }` (nested rule is removed, @at-root content hoisted)
