## ADDED Requirements

### Requirement: Test function declarations SHALL use expanded format
All `#[test]` annotations SHALL be placed on their own line, followed by `fn name()` on the next line.

#### Scenario: Converting compact test declarations
- **WHEN** a file contains `#[test] fn diag_list() { ... }` (e.g., `diagnostic_runner.rs`)
- **THEN** it SHALL be converted to:
  ```rust
  #[test]
  fn diag_list() {
      ...
  }
  ```

#### Scenario: Single-line test body
- **WHEN** a test body is a single expression (e.g., `diag("dir", 15);`)
- **THEN** the function SHALL still use expanded `#[test]` + `fn` format

### Requirement: Test functions SHALL have descriptive comments where appropriate
Complex test functions MAY include a `///` or `//` comment explaining the test purpose.

#### Scenario: Obvious test name
- **WHEN** the test name clearly conveys purpose (e.g., `test_compile_simple`)
- **THEN** no additional comment is required

#### Scenario: Non-obvious test logic
- **WHEN** the test involves non-obvious setup or assertions
- **THEN** a brief comment SHALL be added explaining the intent
