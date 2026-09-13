## ADDED Requirements

### Requirement: Map + Map merge
The `+` operator SHALL merge two Maps, with later keys taking precedence.

#### Scenario: Merge two maps
- **WHEN** `(a: 1, b: 2) + (b: 3, c: 4)` is evaluated
- **THEN** result is `(a: 1, b: 3, c: 4)`

#### Scenario: Merge with overlapping keys
- **WHEN** `($map1: (x: 1)) + ($map2: (x: 2))` is evaluated
- **THEN** `$map2.x` value (`2`) takes precedence

### Requirement: Map + Null identity
The `+` operator SHALL treat Null as identity element for Map.

#### Scenario: Map + Null
- **WHEN** `(a: 1) + null` is evaluated
- **THEN** result is `(a: 1)`

#### Scenario: Null + Map
- **WHEN** `null + (a: 1)` is evaluated
- **THEN** result is `(a: 1)`

### Requirement: Bool + Bool stringification
The `+` operator SHALL concatenate two Booleans as strings.

#### Scenario: true + false
- **WHEN** `true + false` is evaluated
- **THEN** result is `"truefalse"`

### Requirement: Null + Null identity
The `+` operator SHALL return Null for Null + Null.

#### Scenario: Null addition
- **WHEN** `null + null` is evaluated
- **THEN** result is `null`
