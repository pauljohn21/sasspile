# css-serializer Specification

## Purpose
TBD - created by archiving change boost-pass-rate. Update Purpose after archive.
## Requirements
### Requirement: Declaration separator normalization
The CSS serializer MUST output consistent spacing around colons in declarations.

#### Scenario: Property-value spacing
- **WHEN** `a { color: red; }` is compiled
- **THEN** output is exactly `a {\n  color: red;\n}\n` (single space after colon, no space before)

#### Scenario: Multi-value property
- **WHEN** `a { margin: 0 auto; }` is compiled
- **THEN** output preserves space-separated values with single space after colon

### Requirement: Selector list spacing
The CSS serializer MUST output consistent spacing after commas in selector lists.

#### Scenario: Multiple selectors
- **WHEN** `.a, .b { color: red; }` is compiled
- **THEN** output contains `.a, .b {` (single space after comma)

### Requirement: Closing brace newline
The CSS serializer MUST output each closing brace `}` on its own line.

#### Scenario: Nested rule
- **WHEN** `.a { .b { color: red; } }` is compiled
- **THEN** closing brace `}` of `.b` block is followed by newline

### Requirement: @rule spacing
The CSS serializer MUST output consistent spacing in at-rules.

#### Scenario: @media query
- **WHEN** `@media (min-width: 768px) { .a { color: red; } }` is compiled
- **THEN** output contains `@media (min-width: 768px) {` (single space before brace)

#### Scenario: @keyframes
- **WHEN** `@keyframes name { from { opacity: 0; } to { opacity: 1; } }` is compiled
- **THEN** output has `from {` and `to {` with correct indentation

#### Scenario: Empty keyframe block preservation
- **WHEN** `@keyframes name { 0% {} 100% { opacity: 1; } }` is compiled
- **THEN** output SHALL contain `0% {}` block even when empty, matching EP reference output

### Requirement: Pseudo-element SHALL use double-colon format (CSS3)
The CSS serializer SHALL output pseudo-elements using double-colon notation (`::`) consistently, matching CSS3 specification and EP reference output.

#### Scenario: ::before serialization
- **WHEN** `.a::before { content: ""; }` is compiled
- **THEN** output SHALL contain `::before` (double colon), never single colon `:before`

#### Scenario: ::after serialization
- **WHEN** `.a::after { content: ""; }` is compiled
- **THEN** output SHALL contain `::after` (double colon)

#### Scenario: ::placeholder serialization
- **WHEN** `input::placeholder { color: gray; }` is compiled
- **THEN** output SHALL contain `::placeholder` (double colon)

#### Scenario: ::first-line and ::first-letter
- **WHEN** `.a::first-line { }` or `.a::first-letter { }` is compiled
- **THEN** output SHALL preserve double-colon format

#### Scenario: CSS2 pseudo-elements normalized to double-colon
- **WHEN** `.a:before { content: ""; }` is compiled (SCSS source uses CSS2 syntax)
- **THEN** output SHALL normalize to `::before` (CSS3 double-colon format)

