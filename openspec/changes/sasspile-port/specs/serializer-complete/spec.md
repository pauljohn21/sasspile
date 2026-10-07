# Spec Delta

## Purpose

Complete CSS serializer that transforms CssStmt (or CssNode intermediate representation) streams into formatted CSS strings. Supports Expanded, Compressed, and Nested output styles with @media merging, @at-root hoisting, var() normalization, and vendor prefix injection.

## ADDED Requirements

### Requirement: Expanded output style
The serializer SHALL produce indented, newline-separated CSS output in Expanded mode. Each declaration SHALL be on its own line with proper indentation. Rules SHALL have opening brace on the selector line and closing brace on its own line.

#### Scenario: Simple rule expanded
- **WHEN** serializing `Rule(.btn, [Decl(color, red), Decl(margin, 0)])` in Expanded style
- **THEN** produces:
```
.btn {
  color: red;
  margin: 0;
}
```

### Requirement: Compressed output style
The serializer SHALL produce single-line, whitespace-minified CSS output in Compressed mode. No newlines, no extra spaces. Charset comment SHALL be suppressed.

#### Scenario: Compressed output
- **WHEN** serializing `.btn { color: red; margin: 0; }` in Compressed style
- **THEN** produces `.btn{color:red;margin:0;}`

### Requirement: Nested output style
The serializer SHALL produce indented, nested CSS output matching Sass nested style. Same as Expanded for simple rules but with specific handling for deeply nested structures.

#### Scenario: Nested output
- **WHEN** serializing a nested rule in Nested style
- **THEN** produces properly indented nested CSS matching Sass nested output format

### Requirement: @media merging
The serializer SHALL merge adjacent @media blocks with identical queries into a single block. This prevents duplicate @media wrappers in output.

#### Scenario: Adjacent @media merge
- **WHEN** serializing two `@media (min-width: 768px)` blocks back-to-back
- **THEN** produces a single @media block containing rules from both

### Requirement: @at-root hoisting
The serializer SHALL hoist @at-root nodes to the document root level. Nodes that appear after @at-root SHALL be placed at root, not nested within parent rules.

#### Scenario: At-root hoisting
- **WHEN** encountering AtRoot([Rule(.child, [...])]) inside a parent rule
- **THEN** `.child` appears at root level in output

### Requirement: Declaration ordering
The serializer SHALL preserve declaration order as produced by the evaluator. Vendor-prefixed properties (-moz-, -webkit-, -ms-) SHALL follow the standard property.

#### Scenario: appearance normalization
- **WHEN** encountering `appearance: none`
- **THEN** serializes with `-moz-appearance: none` prefix line before it

### Requirement: Comment preservation
The serializer SHALL preserve important comments (`/*! ... */`) in both Expanded and Compressed modes. Regular comments SHALL be suppressed in Compressed mode.

#### Scenario: Important comment
- **WHEN** serializing `/*! MIT License */` in Compressed mode
- **THEN** produces `/*! MIT License */` in output

### Requirement: CSS normalization
The serializer SHALL apply post-processing normalizations: var() null fallback injection, :not() multi-argument wrapping, appearance -moz- prefix injection. Each normalization SHALL be idempotent.

#### Scenario: var() null fallback
- **WHEN** serializing `var(--x, null)`
- **THEN** handles null fallback according to CSS spec

#### Scenario: :not() multi-arg
- **WHEN** serializing `:not(.a, .b)`
- **THEN** wraps as `:not(.a):not(.b)` for broader browser compatibility
