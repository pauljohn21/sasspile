# Spec Delta

## Purpose

Sass module system supporting `@use`, `@forward`, and `@import` directives with proper namespace management, file resolution, and configuration. Phase 2 deliverable — NOT required for Bootstrap 5.3.x compilation (which uses legacy `@import`), but needed for Element-Plus full compatibility.

## ADDED Requirements

### Requirement: @use directive
The system SHALL support `@use "url"` to load a Sass module. Modules SHALL be loaded once and cached. Namespace SHALL be derived from the filename or specified via `as`.

#### Scenario: Basic @use
- **WHEN** evaluating `@use "sass:math"`
- **THEN** the math module is loaded and its functions accessible as `math.func()`

#### Scenario: @use with namespace
- **WHEN** evaluating `@use "theme" as t`
- **THEN** members accessible as `t.$var`, `@include t.mixin()`

### Requirement: @forward directive
The system SHALL support `@forward "url"` to re-export another module's members. `show`/`hide`/`as prefix-*` SHALL work.

#### Scenario: Forward with prefix
- **WHEN** evaluating `@forward "buttons" as btn-*`
- **THEN** members are re-exported with `btn-` prefix

### Requirement: @import (legacy)
The system SHALL support `@import "file"` for legacy SCSS. Files SHALL be resolved via load paths. Imported CSS SHALL be hoisted to the top of output.

#### Scenario: Import resolution
- **WHEN** evaluating `@import "variables"` with include_path="scss/"
- **THEN** finds and loads "scss/_variables.scss"

### Requirement: File resolution
The file resolver SHALL search for files in order: relative to current file, then each load path. SHALL support partials (`_filename`) and index files (`dir/_index.scss` / `dir/index.scss`).

#### Scenario: Partial resolution
- **WHEN** importing `"variables"` from `scss/main.scss`
- **THEN** resolves to `scss/_variables.scss` if exists

### Requirement: Module caching and circular detection
The system SHALL cache loaded modules to prevent re-loading. SHALL detect circular imports and error gracefully.

#### Scenario: Circular detection
- **WHEN** file A imports file B which imports file A
- **THEN** produces a clear error indicating circular dependency

## Phase 2 Note

This spec is Phase 2 because Bootstrap 5.3.x uses only legacy `@import` which is already covered by the Phase 1 pipeline. @use/@forward support is prioritized for Element-Plus compatibility (which uses @use extensively).
