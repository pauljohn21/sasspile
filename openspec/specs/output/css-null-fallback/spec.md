# CssNullFallback Specification

## Purpose

Defines correct serialization of CSS `var()` function fallbacks when the fallback value is the Sass `null` literal, which must not appear in the compiled output.

## Requirements

### Requirement: var() null fallback produces empty fallback

The serializer SHALL output `var(--name, )` when a CSS custom property has `null` as its fallback value in Sass.

#### Scenario: Variable with null fallback in EP source
- **WHEN** the SCSS source declares `var(--el-message-close-size, $fallback)` where `$fallback` resolves to `null`
- **THEN** the compiled CSS SHALL contain `var(--el-message-close-size, 16px)` (using the variable's actual computed value from the EP theme)
- **AND** the output MUST NOT contain the literal string `null`

#### Scenario: Variable with explicit null fallback
- **WHEN** the SCSS source contains `var(--x, null)`
- **THEN** the output SHALL be `var(--x, )` (empty fallback preserved for valid CSS)

#### Scenario: Variable with non-null fallback
- **WHEN** the SCSS source contains `var(--x, 10px)`
- **THEN** the output SHALL be `var(--x, 10px)` (unchanged behavior)
