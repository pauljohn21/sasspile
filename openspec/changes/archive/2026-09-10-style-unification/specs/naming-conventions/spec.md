## ADDED Requirements

### Requirement: Test names SHALL follow defined prefix conventions
Test function names SHALL use one of the three defined prefixes based on their purpose.

#### Scenario: Unit/integration test
- **WHEN** a test verifies specific functionality (e.g., compilation, evaluation)
- **THEN** the name SHALL use `test_<feature>_<descriptor>` (e.g., `test_compile_variable`)

#### Scenario: Diagnostic test
- **WHEN** a test diagnoses failures in a sass-spec subdirectory
- **THEN** the name SHALL use `diag_<dir>` (e.g., `diag_list`, `diag_selector`)

#### Scenario: Statistics/report test
- **WHEN** a test aggregates metrics or generates reports
- **THEN** the name SHALL use `stats_<dir>` or `generate_<report>` (e.g., `stats_math`, `generate_sass_spec_stats`)

### Requirement: Error messages in tests SHALL use standard defaults
`expect()` and `unwrap_or_else()` messages SHALL use `"unexpected failure in test"` as the default standard message.

#### Scenario: Generic test failure
- **WHEN** a test fails due to unexpected error
- **THEN** the message SHALL be `"unexpected failure in test"`

#### Scenario: Chinese-language test context
- **WHEN** a test is part of a Chinese-commented test group (e.g., `interp_test.rs`)
- **THEN` a Chinese message MAY be used if it improves clarity (e.g., `"编译应成功"`)

### Requirement: Source module files SHALL use snake_case naming
All `.rs` file names SHALL use `snake_case` convention.

#### Scenario: Module file
- **WHEN** creating a new source module
- **THEN** the file SHALL be named in snake_case (e.g., `selector_extend.rs`)

#### Scenario: Test file
- **WHEN** creating a new test file
- **THEN** the file SHALL be named in snake_case (e.g., `selector_extend_test.rs`)
