## ADDED Requirements

### Requirement: @use/@import SHALL preserve CSS output order with comments
When `@use` or `@import` rules are mixed with CSS rules and comments, the serializer SHALL preserve the original document order in the output.

#### Scenario: Comment before @use preserved
- **WHEN** the input is `/* before use */ @use "midstream"; a { in: input }`
- **THEN** the output SHALL maintain the comment and CSS rule in original order

#### Scenario: @use then @import with comments preserved
- **WHEN** the input has comments before/after @use and @import
- **THEN** the output SHALL maintain all comments in their original positions

#### Scenario: @use between CSS rules preserves order
- **WHEN** the input is `a { before: 1 } @use "lib"; b { after: 2 }`
- **THEN** the output SHALL maintain `a` rules before `b` rules
