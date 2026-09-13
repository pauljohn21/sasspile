# extend-directive Specification

## Purpose
TBD - created by archiving change boost-pass-rate. Update Purpose after archive.
## Requirements
### Requirement: @extend optional flag
The `@extend` directive SHALL support `!optional` to silently fail when extendee is not found.

#### Scenario: !optional with non-existent selector
- **WHEN** `.a { @extend .nonexistent !optional; }` is compiled
- **THEN** no error is raised, `.a` is output without extending

### Requirement: @extend with complex selector
The `@extend` SHALL work with descendant combinators.

#### Scenario: Extend descendant selector
- **WHEN** `.a .b { color: red; } .c { @extend .b; }` is compiled
- **THEN** output includes `.a .c` with the extended declaration

### Requirement: @extend all modifier
The `@extend` SHALL support `selector-extend($sel, $target, $extender, $optional)` programmatic form.

#### Scenario: Programmatic extend
- **WHEN** `selector-extend(".a", ".b", ".c")` is called
- **THEN** `.c` inherits `.b`'s declarations wherever `.b` appears in `.a`

