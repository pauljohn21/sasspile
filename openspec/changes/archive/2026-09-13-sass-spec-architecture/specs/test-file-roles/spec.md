## ADDED Requirements

### Requirement: HRX file roles are classified during parsing
The HRX parser SHALL classify each file entry into exactly one role: `input`, `output`, `scss-asset`, or `css-asset`. Classification is by filename:
- `input.scss` / `input.sass` → `input`
- `output.css` → `output`
- Other `.scss` / `.sass` → `scss-asset`
- Other `.css` → `css-asset`

#### Scenario: HRX with standard test files classifies correctly
- **WHEN** HRX contains entries: `input.scss`, `output.css`, `_helper.scss`, `imported.css`
- **THEN** they are classified as: input, output, scss-asset, css-asset respectively

### Requirement: Output files are not written to the test VFS
The `output.css` file content SHALL be stored only as the expected comparison string. It SHALL NOT be written to the temporary directory or VFS that the compiler reads from during test execution.

#### Scenario: run_vfs_case writes only input + assets
- **WHEN** a test case is executed via the test harness
- **THEN** `output.css` content is never written to the temp dir; only input, scss-asset, and css-asset files are written

#### Scenario: output.css not importable during test execution
- **WHEN** input.scss contains `@import "output.css";` (erroneous reference)
- **THEN** the compiler either produces an @import url node OR fails to find the file, but never parses output.css as SCSS source

### Requirement: CSS-asset files are available for @import reference
Files classified as `css-asset` SHALL be written to the test VFS so that `@import "foo.css"` can locate them at compilation time.

#### Scenario: css-asset file written to temp dir
- **WHEN** HRX contains `vendor.css` alongside `input.scss`
- **THEN** `vendor.css` is written to the temp directory so `@import "vendor.css";` resolves correctly

### Requirement: Multiple input files in separate HRX directories produce separate cases
Each directory section within an HRX file that contains an `input.scss` (or `input.sass`) SHALL produce an independent test case.

#### Scenario: HRX with two directories yields two cases
- **WHEN** HRX contains `<===> dir1/input.scss ... <===> dir1/output.css ... <===> dir2/input.scss ... <===> dir2/output.css`
- **THEN** the parser produces two HrxCase entries, each with its own input_path and expected_output
