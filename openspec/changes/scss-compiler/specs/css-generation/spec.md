## ADDED Requirements

### Requirement: CSS Generator SHALL emit valid CSS from AST
The generator SHALL walk the AST and emit properly formatted CSS text.

#### Scenario: Simple rule
- **WHEN** generating CSS from `a { color: red; }`
- **THEN** output is exactly `a {\n  color: red;\n}\n`

#### Scenario: Nested rule expansion
- **WHEN** generating CSS from nested SCSS `a { &:hover { color: red; } }`
- **THEN** output is `a:hover {\n  color: red;\n}\n`

#### Scenario: Nested selectors with parent reference
- **WHEN** generating CSS from `nav { a { color: blue; } }`
- **THEN** output is `nav a {\n  color: blue;\n}\n`

### Requirement: CSS Generator SHALL output @media and @supports queries
The generator SHALL correctly emit at-rules with nested content blocks.

#### Scenario: @media query
- **WHEN** generating from `@media (min-width: 768px) { .foo { display: flex; } }`
- **THEN** output preserves nesting inside `@media` block

#### Scenario: @supports with comma selectors
- **WHEN** generating from `@supports (display: grid) { .grid { display: grid; } }`
- **THEN** output is valid CSS with conditional at-rule

### Requirement: CSS Generator SHALL handle @import and @use
The generator SHALL emit appropriate CSS `@import` rules; `@use` disappears in output (its references resolved).

#### Scenario: @import URL
- **WHEN** generating from `@import "foo.css";`
- **THEN** output is `@import "foo.css";\n`

#### Scenario: @use with URL
- **WHEN** generating from `@use "sass:color";`
- **THEN** output is empty (values/functions inlined from module)

### Requirement: CSS Generator SHALL compress output when configured
The generator SHALL support output styles: `Expanded`, `Compressed`, `Compact`, `Nested`.

#### Scenario: Compressed style
- **WHEN** generating with `OutputStyle::Compressed`
- **THEN** no extra whitespace, no newlines, no comments

#### Scenario: Expanded style
- **WHEN** generating with `OutputStyle::Expanded`
- **THEN** indented with 2-space indent and each declaration on its own line

### Requirement: CSS Generator SHALL preserve source maps
The generator SHALL optionally emit source map v3 JSON mapping generated CSS back to source lines.

#### Scenario: Source map presence
- **WHEN** generating with `source_map: true`
- **THEN** returned `SourceMap` contains mappings array with VLQ-encoded entries

#### Scenario: Source map with content
- **WHEN** generating with `source_map: true` and `source_map_include_sources: true`
- **THEN** returned `SourceMap` includes `sourcesContent` with original SCSS text
