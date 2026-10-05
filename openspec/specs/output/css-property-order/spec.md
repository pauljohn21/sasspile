# CssPropertyOrder Specification

## Purpose

Defines consistent CSS property output ordering and value correctness across serializer passes to match dart-sass output for EP components.

## Requirements

### Requirement: CSS properties follow dart-sass ordering

The serializer SHALL output CSS properties in the same order as dart-sass, matching EP baseline output.

#### Scenario: User-select property placement
- **WHEN** SCSS source declares `user-select: none` alongside other properties
- **THEN** it MUST appear at the same position relative to siblings as in the EP baseline

#### Scenario: Variable reference resolution
- **WHEN** a property value references a Sass variable that resolves to a CSS custom property
- **THEN** the output SHALL contain the correct computed value (e.g., `var(--el-color-primary)` not `var(--el-color-disabled)`)

#### Scenario: Background/transition shorthand ordering
- **WHEN** SCSS source has longhand properties like `background-color`, `transition`, `padding`
- **THEN** properties MUST be emitted in the same order as dart-sass processes them

### Requirement: Selector flattening

In BEM-like patterns where dart-sass produces flat selectors (e.g., `.el-popover__title` instead of `.el-popover .el-popover__title`), the serializer SHALL follow dart-sass's nesting-flattening behavior when the parent selector is pure containment (no combinatorial selectors).

#### Scenario: Nested BEM element with pure class parent
- **WHEN** SCSS contains `.el-popover { e(title) { color: red } }`
- **THEN** the output selector SHALL be `.el-popover__title` (flat) when dart-sass produces flat output
- **AND** MUST be `.el-popover .el-popover__title` only when dart-sass produces nested output
