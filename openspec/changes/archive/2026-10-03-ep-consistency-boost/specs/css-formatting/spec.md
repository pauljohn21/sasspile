# Spec: CSS Output Formatting (EP Alignment)

## ADDED Requirements

### Requirement: color.mix Output Format

`color.mix` builtin SHALL output `rgb(r%, g%, b%)` format for mixed colors, matching dart-sass behavior.

#### Scenario: Mix with weight 0.1

- **WHEN** `color.mix(white, #409eff, 10%)` is evaluated
- **THEN** output SHALL be `rgb(77.5294117647%, 88.5882352941%, 100%)` (percent format)

#### Scenario: Mix with legacy space fallback

- **WHEN** `color.mix` result is in legacy RGB space
- **THEN** output SHALL preserve `RgbPercent` mode (not revert to `Auto`)

### Requirement: Selector Interpolation Parent Expansion

`eval_selector_str` SHALL expand `&` inside `#{...}` interpolation to parent selector value.

#### Scenario: BEM element interpolation

- **WHEN** `.el-component { #{& + '__element'} { ... } }` is evaluated
- **THEN** output SHALL be `.el-component__element`

#### Scenario: BEM modifier interpolation

- **WHEN** `.el-component { #{& + '--modifier'} { ... } }` is evaluated
- **THEN** output SHALL be `.el-component--modifier`

### Requirement: At-Root Hoisting Order

`RuleBuilder::build` SHALL place `@at-root` nodes after parent declarations, before nested children.

#### Scenario: Standard at-root placement

- **WHEN** `.x { color: red; @at-root { .y { } } .z { } }` is evaluated
- **THEN** output order: `.x`, `.y`, `.x .z`
