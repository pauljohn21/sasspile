## ADDED Requirements

### Requirement: selector-extend parent unification
`selector-extend` SHALL correctly replace extendee compound within a compound complex selector while preserving parent relationships.

#### Scenario: Simple parent replacement
- **WHEN** calling `selector.extend(".c.x .d", ".c", ".e")`
- **THEN** result SHALL be `.c.x .d, .x.e .d` (`.c` replaced with `.e` in each compound context)

#### Scenario: Complex parent replacement
- **WHEN** calling `selector.extend(".c.x .d", ".c", ".e .f")`
- **THEN** result SHALL be `.c.x .d, .e .x.f .d` (multi-compound extender expands correctly)

#### Scenario: Parent list replacement
- **WHEN** calling `selector.extend(".c.x .d", ".c", ".e, .f")`
- **THEN** result SHALL be `.c.x .d, .x.e .d, .x.f .d` (multiple extenders each generate one variant)

### Requirement: selector-extend grandparent unification
`selector-extend SHALL correctly handle replacement when match occurs within grandparent context.

#### Scenario: Simple grandparent replacement
- **WHEN** calling `selector.extend(".c .d.x .e", ".d", ".f")`
- **THEN** result SHALL be `.c .d.x .e, .c .x.f .e` (grandparent context preserved)

#### Scenario: Complex grandparent replacement
- **WHEN** calling `selector.extend(".c .d.x .e", ".d", ".f .g")`
- **THEN** result SHALL be `.c .d.x .e, .c .f .x.g .e, .f .c .x.g .e` (multi-compound extender with grandparent)

#### Scenario: Grandparent list replacement
- **WHEN** calling `selector.extend(".c .d.x .e", ".d", ".f, .g")`
- **THEN** result SHALL be `.c .d.x .e, .c .x.f .e, .c .x.g .e`

### Requirement: selector-extend leading combinator handling
`selector-extend` SHALL correctly handle extenders with leading combinators.

#### Scenario: Extender with leading child combinator
- **WHEN** calling `selector.extend(".c .d", ".d", "> .e")`
- **THEN** result SHALL include `.c > .e` variant (combinator preserved correctly)

#### Scenario: Extender with leading sibling combinator
- **WHEN** calling `selector.extend(".c .d", ".d", "+ .e")`
- **THEN** result SHALL include `.c + .e` variant
