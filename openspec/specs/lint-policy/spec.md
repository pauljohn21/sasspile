## Requirements

### Requirement: Cargo.toml SHALL define lints section

The project `Cargo.toml` SHALL include a `[lints.rust]` section with `unsafe_code = "warn"` and a `[lints.clippy]` section with `all = "warn"` and `pedantic = "warn"`.

#### Scenario: clippy default passes

- **WHEN** running `cargo clippy --workspace` with no extra flags
- **THEN** the command exits with zero errors and zero warnings

#### Scenario: clippy pedantic passes

- **WHEN** running `cargo clippy --workspace -- -W clippy::pedantic`
- **THEN** the command exits with zero errors and zero warnings (excluding module-level `#![allow(...)]` exemptions)

### Requirement: never_loop error SHALL be fixed

The `strip_parens` function in `src/eval/value/calc.rs` SHALL NOT contain loops that never iterate. The `while` loop SHALL be replaced with an `if` block since the logic only needs one execution.

#### Scenario: strip_parens single layer

- **WHEN** calling `strip_parens("((1px))")`
- **THEN** the function strips one layer of outer parentheses and returns `"(1px)"` (inner parentheses are preserved because the content is not a simple number)

#### Scenario: strip_parens simple number

- **WHEN** calling `strip_parens("(42)")`
- **THEN** the function strips the parentheses and returns `"42"` (simple number detected)

### Requirement: Color modules SHALL use module-level allow for noise lints

Color conversion modules (`color_conv.rs`, `color_adjust.rs`, `color.rs`, `color_names.rs`, `color_types.rs`, `named_colors.rs`) SHALL use `#![allow(...)]` for lints that conflict with standard color science naming conventions: `single_char_names`, `unreadable_literal`, `excessive_precision`.

#### Scenario: color module clippy clean

- **WHEN** running `cargo clippy --workspace -- -W clippy::pedantic`
- **THEN** no warnings are emitted from color conversion modules for `single_char_names`, `unreadable_literal`, or `excessive_precision`

### Requirement: Cast operations SHALL use safe conversions

All `as` cast operations in non-color modules SHALL be replaced with safe conversion methods: `u8::try_from`, `f64::from`, `usize::try_from`, or `i64::from` as appropriate. Color modules MAY use `as u8` with clamp for RGB component clamping.

#### Scenario: no cast warnings in non-color code

- **WHEN** running `cargo clippy --workspace -- -W clippy::pedantic`
- **THEN** no `cast_truncation`, `cast_sign_loss`, `cast_possible_truncation`, `cast_possible_wrap`, or `cast_precision_loss` warnings appear for non-color source files

### Requirement: Documentation SHALL include backticks for technical terms

All `///` doc comments SHALL wrap technical terms (function names, type names, keywords, paths) in backticks. Functions returning `Result` SHALL include a `# Errors` section. Functions that may panic SHALL include a `# Panics` section.

#### Scenario: no doc_markdown warnings

- **WHEN** running `cargo clippy --workspace -- -W clippy::pedantic`
- **THEN** no `doc_markdown` warnings are emitted

#### Scenario: no missing_errors_doc warnings

- **WHEN** running `cargo clippy --workspace -- -W clippy::pedantic`
- **THEN** no `missing_errors_doc` warnings are emitted

### Requirement: No wildcard imports in non-test code

All `use` statements in `src/` SHALL use explicit imports rather than wildcards (`use x::*`), except where a wildcard import is necessary for macro-generated code or trait implementation.

#### Scenario: no wildcard_imports warnings

- **WHEN** running `cargo clippy --workspace -- -W clippy::pedantic`
- **THEN** no `wildcard_imports` warnings are emitted for `src/` code

### Requirement: No test regression after lint fixes

All existing tests SHALL continue to pass after lint fixes: `compile_test` (43), `stage_test` (10), `ast_test` (8), `common_test` (5), `interp_test` (15), `bs_spec` (15), `ep_full` (121), `default_config_test` (9). The sass-spec pass rate SHALL NOT decrease below 3216/5624.

#### Scenario: core tests pass

- **WHEN** running `cargo test --test compile_test --test stage_test --test ast_test --test common_test --test interp_test --test bs_spec --test ep_full --test default_config_test -- --test-threads=1`
- **THEN** all 226 tests pass

#### Scenario: sass-spec baseline maintained

- **WHEN** running `cargo test --test sass_spec_full`
- **THEN** the pass count is at least 3216
