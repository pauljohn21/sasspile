## ADDED Requirements

### Requirement: @import of CSS files produces pass-through @import rule
When SCSS source contains `@import "foo.css"` (or any URL ending in `.css`, `http://`, `https://`, `//`, `url(`, or with a media modifier), the compiler SHALL output a `@import` CSS node with the appropriate `url()` or quoted string form, and SHALL NOT attempt to parse the target file as SCSS.

#### Scenario: @import "foo.css" outputs @import url("foo.css")
- **WHEN** SCSS contains `@import "foo.css";`
- **THEN** output is `@import url("foo.css");` and no file I/O occurs for `foo.css`

#### Scenario: @import "foo.css" screen outputs with modifier
- **WHEN** SCSS contains `@import "foo.css" screen;`
- ****THEN** output is `@import "foo.css" screen;`

#### Scenario: @import "http://example.com/foo.css" uses url() form
- **WHEN** SCSS contains `@import "http://example.com/foo.css";`
- **THEN** output is `@import url("http://example.com/foo.css");`

#### Scenario: @import of .scss file still goes through SCSS pipeline
- **WHEN** SCSS contains `@import "foo.scss";` or `@import "foo";` (resolves to `foo.scss`)
- **THEN** the target file is loaded, parsed as SCSS, and its declarations are inlined

### Requirement: @use of CSS file produces a clear error
The compiler SHALL reject `@use "foo.css"` with an explicit error indicating CSS files cannot be used as modules.

#### Scenario: @use "foo.css" errors
- **WHEN** SCSS contains `@use "foo.css";`
- **THEN** compilation fails with an error message indicating CSS files can't be @used

### Requirement: @forward of CSS file produces a clear error
The compiler SHALL reject `@forward "foo.css"` with an explicit error indicating CSS files cannot be forwarded as modules.

#### Scenario: @forward "foo.css" errors
- **WHEN** SCSS contains `@forward "foo.css";`
- **THEN** compilation fails with an error message indicating CSS files can't be @forwarded

### Requirement: import resolver does not load .css files through SCSS pipeline
The `load_import` function SHALL only be invoked for SCSS/SASS files. When a `.css` file is resolved via path, the eval_import function SHALL return before calling `load_import`, emitting only the CSS `@import` node.

#### Scenario: resolve_file_import returns .css path does not reach load_import
- **WHEN** `@import "existing.css";` resolves via `resolve_file_import` to a `.css` path
- **THEN** `eval_import` returns early with a `@import url(...)` node and never calls `load_import`
