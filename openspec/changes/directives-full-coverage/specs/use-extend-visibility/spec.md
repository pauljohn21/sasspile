## ADDED Requirements

### Requirement: @use imported selectors SHALL be visible to @extend
Selectors defined in a `@use`d module SHALL be extendable from the importing file, except for private selectors (prefixed with `-` or `%`).

#### Scenario: Basic cross-module extend
- **WHEN** `_lib.scss` defines `.in-lib { color: red }` and `input.scss` does `@use "lib"; .in-input { @extend .in-lib; }`
- **THEN** the output SHALL include `.in-input` in `.in-lib`'s selector group

#### Scenario: Private selector cannot be extended
- **WHEN** `_lib.scss` defines `%-private { color: red }` and `input.scss` does `@use "lib"; .in-input { @extend %-private !optional; }`
- **THEN** the output SHALL NOT error (with !optional) and `.in-input` remains unchanged

#### Scenario: Diamond dependency extend merge
- **WHEN** two @use'd modules both define the same selector and input extends it
- **THEN** the output SHALL merge all three selector groups correctly

#### Scenario: Midstream extend within pseudoselector
- **WHEN** `_upstream.scss` defines `.in-upstream`, `_midstream.scss` does `:is(.in-midstream) { @extend .in-upstream }`, and input does `@use "midstream"; .in-input { @extend .in-midstream; }`
- **THEN** the output SHALL correctly resolve the transitive extend

#### Scenario: Extend scope through import into use
- **WHEN** a module is @use'd and also @import'd, and extend crosses the boundary
- **THEN** the output SHALL correctly resolve the extend across both import mechanisms
