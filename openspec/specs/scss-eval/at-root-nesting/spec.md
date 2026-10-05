# AtRootNesting Specification

## Purpose

Defines correct `@at-root` semantics when processing nested BEM mixin invocations (`when()`, `e()`, `m()`) where the at-root body contains Rules whose selectors must preserve their parent context.

## Requirements

### Requirement: outer_only must preserve nested Rule structure

`eval_at-root` MUST NOT unwrap the wrapper Rule when `outer_only` is TRUE and the wrapper's body contains Rule descendants. This additionally applies when the wrapper's selector is an interpolated `#{$variable}` expression or when hitAllSpecialNestRule paths produce 3+ levels of nesting.

#### Scenario: when() + e() compound modifier with nested element

- **WHEN** `e()` mixin is invoked inside `when()` mixin's `@content` and `hitAllSpecialNestRule` is true
- **THEN** the wrapper selector (e.g., `.el-dialog.is-draggable`) is preserved as a prefix for descendant Rules
- **AND** output selector combines wrapper + descendant (e.g., `.el-dialog.is-draggable .el-dialog__header`)

#### Scenario: outer_only with pure declarations

- **WHEN** `outer_only` is TRUE and the wrapper body consists only of declarations (no Rule nodes)
- **THEN** unwrap is permitted — output declarations are placed at root level under the parent selector

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

### Requirement: at-root unconditionally resets selector_chain

`eval_at-root` MUST reset `env.selector_chain` to `parent_sel` regardless of whether the `@at-root` directive has an explicit selector argument.

#### Scenario: nested when() inside e() inside e()

- **WHEN** `when()` mixin is invoked within `e()` content which itself is within another `e()` content
- **THEN** the inner `@at-root` resets chain to immediate parent selector only (not full descendant chain)
- **AND** `&` inside inner `@at-root` expands to the immediate parent (e.g., `.el-collapse-item__arrow`) not the full chain (e.g., `.el-collapse-item__header .el-collapse-item__arrow`)

#### Scenario: at-root with explicit selector argument

- **WHEN** `@at-root(.foo)` has explicit selector
- **THEN** chain is reset to `.foo` (existing behavior, unchanged)
