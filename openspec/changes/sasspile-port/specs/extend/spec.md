# Spec Delta

## Purpose

Sass @extend directive supporting complex selector inheritance, placeholder selectors (%), and media-scope limitations. Phase 2 deliverable — Bootstrap 5.3.x uses @extend in specific patterns (e.g., `.btn-extend`), but the full selector unification algorithm is needed for sass-spec compliance.

## ADDED Requirements

### Requirement: Basic @extend
The system SHALL support `@extend selector` to inherit styles from another selector. The extending selector SHALL be added to the target selector's rule group.

#### Scenario: Simple extend
- **WHEN** evaluating `.error { color: red; } .login-error { @extend .error; }`
- **THEN** output groups `.error, .login-error { color: red; }`

### Requirement: Placeholder @extend
The system SHALL support `@extend %placeholder` where placeholder selectors (%) are only used for extension and do not appear in final CSS.

#### Scenario: Placeholder extend
- **WHEN** evaluating `%btn-style { display: inline-block; } .btn { @extend %btn-style; }`
- **THEN** output is `.btn { display: inline-block; }` (%btn-style not in output)

### Requirement: Complex selector extend
The system SHALL support extending compound selectors, descendant combinators, and pseudo-classes. Selector unification and subset matching SHALL follow sass-spec rules.

#### Scenario: Compound extend
- **WHEN** extending `.nav .item.active` from `.highlight`
- **THEN** .highlight is grouped with .nav .item.active

### Requirement: @extend within @media
The system SHALL only allow @extend to target selectors within the same @media scope. Extending across media boundaries SHALL produce an error.

#### Scenario: Cross-media extend error
- **WHEN** `@media screen { @extend .print-only }` where `.print-only` is outside `@media`
- **THEN** produces an error about cross-media extension

### Requirement: Optional extend
The system SHALL support `@extend selector !optional` which silently does nothing if the target selector doesn't exist.

#### Scenario: Optional non-existent target
- **WHEN** evaluating `.a { @extend .nonexistent !optional; }`
- **THEN** produces no error, .a remains unchanged

## Phase 2 Note

This spec is Phase 2 because while Bootstrap uses @extend in limited patterns, the full @extend implementation (selector unification, compound matching, media-scope awareness) is complex. Phase 1 should handle basic extend that Bootstrap needs; the full sass-spec compliance version is Phase 2.
