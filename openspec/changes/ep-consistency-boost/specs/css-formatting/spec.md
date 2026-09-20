# Spec: CSS Output Format Normalization

## ADDED Requirements

### Requirement: Named Colors Mapped to Hex

The serializer SHALL output CSS using dart-sass-compatible color representations.

#### Scenario: `white` becomes hex

- **WHEN** CSS variable or declaration value contains named color `white`
- **THEN** serialized output SHALL use `#ffffff`

#### Scenario: `black` becomes hex

- **WHEN** CSS variable or declaration value contains named color `black`
- **THEN** serialized output SHALL use `#000000`

#### Scenario: `rgba(0, 0, 0, 0)` becomes `transparent`

- **WHEN** color expression evaluates to fully transparent black
- **THEN** serialized output SHALL use `transparent`

### Requirement: CSS Function Names Use Standard Casing

#### Scenario: `scalex()` → `scaleX()`

- **WHEN** transform function `scalex(N)` appears in output
- **THEN** serialized output SHALL be `scaleX(N)`

#### Scenario: `translatex()` → `translateX()`

- **WHEN** transform function `translatex(N)` appears in output
- **THEN** serialized output SHALL be `translateX(N)`

#### Scenario: `rotatez()` → `rotateZ()`

- **WHEN** transform function `rotatez(N)` appears in output
- **THEN** serialized output SHALL be `rotateZ(N)`

### Requirement: Keyframe Percentage NOT Escaped

#### Scenario: `0%` in @keyframes not escaped

- **WHEN** serializing `@keyframes` rule content with percentage `0%`
- **THEN** output SHALL be `0%` (not `\30 0\%`)
