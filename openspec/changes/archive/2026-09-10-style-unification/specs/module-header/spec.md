## ADDED Requirements

### Requirement: Every file SHALL have a module header
Every `.rs` file in `src/` and `tests/` SHALL begin with a `//!` module-level doc comment containing at minimum a one-line summary.

#### Scenario: New src module created
- **WHEN** a developer creates `src/eval/new_module.rs`
- **THEN** the file SHALL start with `//! —— Module Name ——` followed by `//!` summary lines

#### Scenario: New test file created
- **WHEN** a developer creates `tests/new_test.rs`
- **THEN** the file SHALL start with `//! —— Test Description ——` followed by `//!` summary lines and a `## Coverage Scenarios` section

#### Scenario: File currently without header
- **WHEN** the style unification task processes a file that currently has no `//!` header (e.g., `tests/ast_test.rs`)
- **THEN** the task SHALL add the appropriate template header

### Requirement: src module header SHALL follow structured format
Every `src/` module file SHALL follow the structured header format with optional sections for concepts and examples.

#### Scenario: src module with public API
- **WHEN** a `src/` module exposes public types/functions
- **THEN** the header SHALL include a `## Core Concepts` section listing key types and their relationships

#### Scenario: src module with simple internal logic
- **WHEN** a `src/` module contains only internal helpers
- **THEN** the header SHALL include at minimum a one-line summary and MAY omit the `## Core Concepts` section

### Requirement: Test file header SHALL include coverage scenarios
Every `tests/` module file SHALL include a `## Coverage Scenarios` section listing what is tested.

#### Scenario: Unit test file
- **WHEN** a test file covers specific functionality
- **THEN** the header SHALL list the scenarios in bullet form under `## Coverage Scenarios`

#### Scenario: Integration test file
- **WHEN** a test file covers end-to-end compilation
- **THEN** the header SHALL include a `## sass-spec Reference` section linking to relevant spec directories
