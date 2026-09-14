# use-css-position-preservation

## Requirements

### Requirement: @use-generated CSS at-rules maintain source position

WHEN `@use "url"` generates CSS at-rules (e.g., `@import`), those at-rules SHALL appear at the same position in the output CSS as the `@use` statement in the source SCSS.

#### Scenario: @use between rules

- GIVEN source SCSS: `.a {} @use "lib"; .b {}`
- WHEN compiled to CSS
- THEN output is `.a {} @import "lib"; .b {}` (order preserved)

#### Scenario: Multiple @use statements preserve order

- GIVEN source SCSS: `@use "a"; .x {} @use "b"; .y {}`
- WHEN compiled
- THEN output order: `@import "a"; .x {} @import "b"; .y {}`
