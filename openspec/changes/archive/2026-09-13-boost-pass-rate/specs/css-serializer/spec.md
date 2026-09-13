## ADDED Requirements

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
