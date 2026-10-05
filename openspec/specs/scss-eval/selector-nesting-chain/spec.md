# scss-eval/selector-nesting-chain Specification

## Purpose

Defines correct selector nesting and composition behavior for SCSS evaluator, ensuring mixin outputs and interpolation-derived selectors are properly combined with their parent context.

## Requirements

### Requirement: Mixin output in nested context SHALL be descendant-combined with parent

#### Scenario: b() mixin inside nested rule
- **WHEN** `b(scrollbar__bar)` mixin is included inside `.el-popper { }`
- **THEN** output selector is `.el-popper .el-scrollbar__bar`

#### Scenario: #{& + '-suffix'} interpolation descendant combine
- **WHEN** `#{& + '-selfdefine'}` rule is nested inside `.el-popper { }`
- **THEN** output selector is `.el-popper .el-popper-selfdefine`

### Requirement: BEM compound prefix detection SHALL only match -- and __ separators

#### Scenario: BEM modifier is compound
- **WHEN** parent is `.el-button` and child is `.el-button--large`
- **THEN** `starts_with_compound_prefix` returns `true`

#### Scenario: BEM element is compound
- **WHEN** parent is `.el-button` and child is `.el-button__inner`
- **THEN** `starts_with_compound_prefix` returns `true`

#### Scenario: Hyphen-suffix class with interpolation is NOT compound
- **WHEN** parent is `.el-popper` and child is `.el-popper-selfdefine` (from `#{& + '-selfdefine'}`)
- **THEN** `starts_with_compound_prefix` returns `false`

#### Scenario: Single-underscore suffix is NOT compound
- **WHEN** parent is `.el_btn` and child is `.el_btn_self`
- **THEN** `starts_with_compound_prefix` returns `false`

### Requirement: Structural selector characters SHALL remain compound separators

#### Scenario: Compound via class dot
- **WHEN** parent is `.el-button` and child is `.el-button.is-large`
- **THEN** `starts_with_compound_prefix` returns `true`

#### Scenario: Compound via pseudo-class
- **WHEN** parent is `.el-button` and child is `.el-button:first-child`
- **THEN** `starts_with_compound_prefix` returns `true`

#### Scenario: Compound via child combinator
- **WHEN** parent is `.el-button` and child is `.el-button > .el-icon`
- **THEN** descendant nesting applies (child already has parent prefix handled by combinator)
