## ADDED Requirements

### Requirement: selector-nest with parent reference
`selector-nest($parent1, $parent2, $child)` SHALL correctly handle `&` parent references.

#### Scenario: Single parent with child
- **WHEN** `selector-nest(".a", ".b .c")` is evaluated
- **THEN** result equals `(".a .b" ".a .c")` (selector list)

#### Scenario: Nested with ampersand
- **WHEN** `selector-nest(".a", "&.b")` is evaluated inside selector context
- **THEN** result is `".a.b"`

### Requirement: selector-append compound
`selector-append($selector1, $selector2)` SHALL handle compound selectors (class + pseudo).

#### Scenario: Append class to compound
- **WHEN** `selector-append(".a:hover", ".b")` is evaluated
- **THEN** result equals `(".a:hover.b")`

#### Scenario: Append universal selector
- **WHEN** `selector-append(".a", "*")` is evaluated
- **THEN** result equals `(".a")` (universal is redundant)

### Requirement: selector-unify class and type
`selector-unify($selector1, $selector2)` SHALL unify class with type selector.

#### Scenario: Type and class unification
- **WHEN** `selector-unify("div", ".foo")` is evaluated
- **THEN** result is `div.foo` (combined)

#### Scenario: Non-unifiable selectors
- **WHEN** `selector-unify(".a", ".b")` is evaluated (no shared base)
- **THEN** result is `null`

### Requirement: selector-extend complex
`selector-extend($selector, $extendee, $extender)` SHALL handle complex selector combinators.

#### Scenario: Extend with descendant
- **WHEN** `selector-extend(".a .b", ".a", ".c")` is evaluated
- **THEN** result is `(".a .b" ".c .b")`

#### Scenario: Extend compound
- **WHEN** `selector-extend(".a.b", ".a", ".c")` is evaluated
- **THEN** result equals `(".a.b" ".c.b")`
