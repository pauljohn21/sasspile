## ADDED Requirements

### Requirement: round() accepts strategy argument
The `round()` function SHALL accept a 2-3 argument form `round(strategy, number, step?)` conforming to the CSS round() specification, where `strategy` is one of "up", "down", "nearest", "to-zero".

#### Scenario: round with 2 arguments (nearest strategy, no step)
- **WHEN** `round("nearest", 5.5)` is evaluated
- **THEN** the result is `6`

#### Scenario: round up with positive number
- **WHEN** `round("up", 5.1)` is evaluated
- **THEN** the result is `6`

#### Scenario: round down with positive number
- **WHEN** `round("down", 5.9)` is evaluated
- **THEN** the result is `5`

#### Scenario: round to-zero with negative number
- **WHEN** `round("to-zero", -5.9)` is evaluated
- **THEN** the result is `-5`

#### Scenario: round up with negative number
- **WHEN** `round("up", -5.1)` is evaluated
- **THEN** the result is `-5` (up = toward +infinity)

#### Scenario: round down with negative number
- **WHEN** `round("down", -5.1)` is evaluated
- **THEN** the result is `-6` (down = toward -infinity)

#### Scenario: round with 3 arguments (step)
- **WHEN** `round("nearest", 5px, 2px)` is evaluated
- **THEN** the result is `6px` (rounds to nearest multiple of 2)

#### Scenario: round down with step
- **WHEN** `round("down", 7px, 3px)` is evaluated
- **THEN** the result is `6px` (largest multiple of 3 <= 7)

#### Scenario: round up with step
- **WHEN** `round("up", 5px, 3px)` is evaluated
- **THEN** the result is `6px` (smallest multiple of 3 >= 5)

#### Scenario: round with infinity
- **WHEN** `round("down", infinity)` is evaluated
- **THEN** the result is `infinity`

#### Scenario: round with negative infinity
- **WHEN** `round("up", -infinity)` is evaluated
- **THEN** the result is `-infinity`

#### Scenario: 1-argument form preserved
- **WHEN** `round(5.5)` is evaluated (existing behavior)
- **THEN** the result is `6` (backward-compatible)

#### Scenario: wrong argument count rejected
- **WHEN** `round()` is called with 0 arguments
- **THEN** an error `Missing argument $number.` is raised

#### Scenario: invalid strategy rejected
- **WHEN** `round("invalid", 5)` is evaluated
- **THEN** an error is raised indicating invalid strategy

### Requirement: round() step validation
The `round()` function SHALL validate that the number and step have compatible units.

#### Scenario: round with incompatible units
- **WHEN** `round("nearest", 5px, 2em)` is evaluated
- **THEN** an error about incompatible units is raised

#### Scenario: round with zero step
- **WHEN** `round("nearest", 5px, 0px)` is evaluated
- **THEN** an error about step cannot be zero is raised

#### Scenario: round with unitless number and unit step
- **WHEN** `round("nearest", 5, 2px)` is evaluated
- **THEN** an error about incompatible units is raised

### Requirement: round() with calc() preservation
The `round()` function SHALL preserve `calc()` expressions when strategy is absent or inputs contain calc.

#### Scenario: round with calc number
- **WHEN** `round(calc(2px + 3px))` is evaluated (1-arg)
- **THEN** the result is `round(calc(2px + 3px))` (string preservation)
