## ADDED Requirements

### Requirement: STYLE_GUIDE.md SHALL exist at project root
The project SHALL have a standalone style guide document at `/STYLE_GUIDE.md` that defines all code style conventions for both `src/` and `tests/` directories.

#### Scenario: New contributor writes a new test file
- **WHEN** a new contributor creates a new `tests/my_feature_test.rs`
- **THEN** they SHALL consult `STYLE_GUIDE.md` for the correct module header template, section divider format, and test function declaration style

#### Scenario: STYLE_GUIDE.md covers all 5 dimensions
- **WHEN** a reviewer checks `STYLE_GUIDE.md` for completeness
- **THEN** the document SHALL cover: module header templates, section dividers, function declaration format, error message conventions, and naming conventions

### Requirement: STYLE_GUIDE.md SHALL define module header templates
The style guide SHALL provide two distinct module header templates: one for `src/` modules and one for `tests/` files.

#### Scenario: src module header
- **WHEN** adding a module header to a `src/eval/new_module.rs`
- **THEN** the header SHALL follow the src template: `//!` title + one-line summary + `## Core Concepts` section + optional `## Example`

#### Scenario: test file header
- **WHEN** adding a module header to a `tests/new_feature_test.rs`
- **THEN** the header SHALL follow the test template: `//!` title + one-line summary + `## Coverage Scenarios` + optional `## sass-spec Reference`

### Requirement: STYLE_GUIDE.md SHALL define section divider convention
The style guide SHALL specify a single section divider character and alignment standard.

#### Scenario: Section divider in source file
- **WHEN** a developer separates logical sections in a file
- **THEN** they SHALL use `// ─── Section Name ───────────────────────────` (U+2500, right-padded to 78 columns)

### Requirement: STYLE_GUIDE.md SHALL define test function declaration format
The style guide SHALL mandate expanded `#[test]\nfn` format without exception.

#### Scenario: Single-line test function
- **WHEN** a developer writes a test that fits on one line (e.g., `diag("dir", 15);`)
- **THEN** they SHALL still use expanded format with `#[test]` on its own line

### Requirement: STYLE_GUIDE.md SHALL define naming conventions
The style guide SHALL define the three accepted test name prefixes and their usage context.

#### Scenario: Naming a unit test
- **WHEN** naming a test that verifies a single function's behavior
- **THEN** the name SHALL use the `test_<feature>_<case>` pattern

#### Scenario: Naming a diagnostic test
- **WHEN** naming a test that diagnoses a sass-spec subdirectory
- **THEN** the name SHALL use the `diag_<dir>` pattern

#### Scenario: Naming a statistics test
- **WHEN** naming a test that aggregates stats
- **THEN** the name SHALL use the `stats_<dir>` or `generate_<report>` pattern
