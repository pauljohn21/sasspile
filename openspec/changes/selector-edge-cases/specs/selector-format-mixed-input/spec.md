## ADDED Requirements

### Requirement: value_to_selector_format mixed list support
`value_to_selector_format` SHALL accept a list containing both strings and nested lists as valid selector format input.

#### Scenario: Comma-separated complexes with string and list mixed
- **WHEN** passing `(c, d e)` (string + space-separated list mixed)
- **THEN** result SHALL be `[["c"], ["d", "e"]]` (string becomes single-compound complex, list becomes multi-compound complex)

#### Scenario: Single string in list context
- **WHEN** passing `("c")` as a single-element comma list
- **THEN** result SHALL be `[["c"]]` (single string treated as single compound complex)

#### Scenario: Mixed three elements
- **WHEN** passing `(a, b c, d e f)`
- **THEN** result SHALL be `[["a"], ["b", "c"], ["d", "e", "f"]]`

### Requirement: selector-nest with mixed format input
`selector-nest` SHALL accept mixed list format for any of its arguments.

#### Scenario: First argument mixed
- **WHEN** calling `selector.nest((c, d e), "f")`
- **THEN** result SHALL be `c f, d e f`

#### Scenario: Last argument mixed
- **WHEN** calling `selector.nest("c", (d, e f))`
- **THEN** result SHALL be `c d, c e f`

#### Scenario: Both arguments mixed
- **WHEN** calling `selector.nest((a, b), (c, d e))`
- **THEN** result SHALL be `a c, a d e, b c, b d e`
