# Spec Delta

## Purpose

Full Bootstrap 5.3.x end-to-end compilation validation — the primary acceptance test for the sasspile-port migration. Compiles the complete Bootstrap SCSS project and produces CSS output that matches the official distribution byte-for-byte (within documented tolerances).

## ADDED Requirements

### Requirement: Bootstrap main file compilation
The compiler SHALL successfully compile `bootstrap/scss/bootstrap.scss` (the main entry point) without errors. Output SHALL be greater than 100,000 bytes of CSS.

#### Scenario: Bootstrap expanded compilation
- **WHEN** compiling `bootstrap/scss/bootstrap.scss` with Expanded style and include_path "bootstrap/scss/"
- **THEN** output is valid CSS >100KB containing ".btn", ".container", ".modal", ".navbar" selectors

#### Scenario: Bootstrap compressed compilation
- **WHEN** compiling with Compressed style
- **THEN** output is >50KB with no newline characters

### Requirement: Bootstrap component isolation
The compiler SHALL compile individual Bootstrap component files (reboot, alert, badge, buttons, forms, grid, modal, navbar, functions, variables, maps, mixins) without errors.

#### Scenario: Individual component compilation
- **WHEN** compiling `bootstrap/scss/_buttons.scss` with mixins and variables available
- **THEN** produces valid CSS with .btn-* selectors

### Requirement: Byte-level output matching
The compiler's output for `bootstrap.scss` SHALL match the official Bootstrap 5.3.x dist CSS byte-for-byte within documented floating-point tolerances (≤ 0.001 difference in color channel values).

#### Scenario: Color serialization match
- **WHEN** Bootstrap computes `darken(#0d6efd, 7.5%)` for `.btn-primary:hover`
- **THEN** the resulting color in output matches the dist CSS computed value
