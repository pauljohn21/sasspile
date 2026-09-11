## ADDED Requirements

### Requirement: CSS @import from @use'd modules SHALL be hoisted to output top
When a `@use`'d module contains CSS `@import` statements, those imports SHALL be collected and hoisted to the top of the final CSS output in dependency order.

#### Scenario: Simple use chain with CSS imports
- **WHEN** `input.scss` uses `@use "midstream"` and midstream has `@import "midstream.css"`
- **THEN** the output SHALL contain `@import "midstream.css";` before any rule content

#### Scenario: Diamond dependency with CSS imports
- **WHEN** `input.scss` uses both `@use "left"` and `@use "right"`, and both use `@use "upstream"`
- **THEN** `@import "upstream.css"` SHALL appear only once and before `@import "left.css"` and `@import "right.css"`

#### Scenario: Mixed @use and @import with CSS imports
- **WHEN** `input.scss` has `@use "midstream"` then `@import "input.css"`
- **THEN** the output SHALL have module CSS imports first (`@import "upstream.css"`, `@import "midstream.css"`), then `@import "input.css"`, then rule content

#### Scenario: Nested @import within @use'd module
- **WHEN** `midstream.scss` has `@import "imported"` (another SCSS file) which itself has `@import "imported.css"`
- **THEN** `@import "imported.css"` SHALL be hoisted above `@import "used.css"`

#### Scenario: Comments preserved with imports
- **WHEN** a `@use`'d module has comments before CSS imports
- **THEN** comments SHALL be preserved in output near their associated import (implementation may vary)
