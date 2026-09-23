## Capability: selector-extend

### ADDED Requirements

#### Requirement: Extend placeholder collection
The compiler SHALL collect `@extend %placeholder` and apply it to all rules matching `%placeholder`.

#### Scenario: Basic extend
- **WHEN** "%foo { color: red; } .bar { @extend %foo; }"
- **THEN** output contains ".bar" with "color: red" (inherited from %foo)
- **AND** "%foo" placeholder rule is NOT emitted directly

#### Scenario: Multiple extenders
- **WHEN** ".a, .b { @extend %foo; }"; "%foo { color: red; }"
- **THEN** both ".a" and ".b" receive "color: red"

#### Scenario: Extend within mixin body
- **WHEN** "@mixin pad { @extend %spacer; }" then "@include pad;"
- **THEN** extend rules propagate outside mixin invocation context

#### Scenario: Extend with nested suffix
- **WHEN** "@extend %btn !optional"; ".el-btn--primary { @extend %btn; }"
- **THEN** output ".el-btn--primary" inherits from "%btn" safely

---

### Requirement: Extend state machine integration
`CompileState` SHALL maintain `extends_queued: Vec<(String, String)>` for deferred extend application.

#### Scenario: Deferred application
- During scan_map, `@extend %target` pushes (current_selector, "%target") onto extends_queued
- During finalize_collecting (rule close) or post-processing, queued extends apply

#### Scenario: Apply extends on rule finalization
- When "}" closes a rule with extenders, append extenders' declarations to the rule's children
