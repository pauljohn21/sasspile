# Spec Delta

## Purpose

Extends `@at-root` correct-nesting spec to cover additional BEM mixin patterns where intermediate wrapper levels must be preserved in the selector output.

## MODIFIED Requirements

### Requirement: outer_only must preserve nested Rule structure (EXTENDED)

`eval_at-root` MUST NOT unwrap the wrapper Rule when `outer_only` is TRUE and the wrapper's body contains Rule descendants. This additionally applies when the wrapper's selector is an interpolated `#{$variable}` expression or when hitAllSpecialNestRule paths produce 3+ levels of nesting.

#### Scenario: 3-level deep e() nesting (anchor pattern)
- **WHEN** `e()` is invoked inside `e()` inside `b()` and the middle-level wrapper has descendant Rules
- **THEN** ALL intermediate wrapper levels are preserved (e.g., `.el-anchor--horizontal .el-anchor__list .el-anchor__item`)
- **AND** no intermediate level is dropped

#### Scenario: modifier prefix with multiple children (descriptions pattern)
- **WHEN** `@include m(large)` produces `--large` modifier with multiple child Rules (`e(title)`, `e(cell)`)
- **THEN** the modifier prefix wraps the component child (e.g., `.el-descriptions--large .el-descriptions__header .el-descriptions__title`)
- **AND** intermediate component-child level (`__header`) is preserved

#### Scenario: compound parent with descendant (color-picker-panel pattern)
- **WHEN** `.is-disabled` prefix is combined with `e(color-selector)` via nested at-root
- **THEN** the full compound selector chain is output (e.g., `.is-disabled .el-color-predefine .el-color-predefine__color-selector`)
