# import-css-nesting

## Requirements

### Requirement: @import nested in CSS rule expands inline

WHEN `@import "file"` appears inside a CSS rule, the imported file's content SHALL be expanded at that position.

#### Scenario: Basic nested import

- GIVEN source SCSS: `a { @import "other"; }`
- GIVEN `_other.scss`: `b { c: d; }`
- WHEN compiled
- THEN output is `a { b { c: d; } }` (or equivalent flattened form)

#### Scenario: Nested import with surrounding declarations

- GIVEN source SCSS: `a { color: red; @import "other"; font-size: 16px; }`
- WHEN compiled
- THEN output preserves the order: color, imported content, font-size
