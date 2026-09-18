# each-for-control Specification

## ADDED Requirements

### Requirement: @each $x in list expands loop body

`@each $x in (a, b, c)` SHALL expand the loop body as a sub-stream N times,每次 `$x` 绑定到对应元素,产出多个 Rule/Declaration。

#### Scenario: @each 产生多条规则
- **WHEN** 输入 `@each $c in (red, green, .a) { .#{$c} { color: $c; } }`
- **THEN** 输出包含 `.red{color:red}` `.green{color:green}` `.a{color:a}` 三条规则(line order preserved)

### Requirement: @for $i from 1 through N expands

`@for $i from 1 through 3` SHALL iterate 3 times with `$i` = 1, 2, 3 and expand the loop body.

#### Scenario: @for 生成多列
- **WHEN** `@for $i from 1 through 12 { .col-#{$i} { width: $i / 12 * 100%; } }`
- **THEN** 输出 12 条规则 `.col-1` ~ `.col-12`
