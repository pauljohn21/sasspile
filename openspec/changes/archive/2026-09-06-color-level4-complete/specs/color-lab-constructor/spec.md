## ADDED Requirements

### Requirement: lab() constructor
The system SHALL support the `lab(L% a b [/ alpha])` CSS Color 4 constructor that creates a color in CIE Lab color space.

#### Scenario: Basic lab construction
- **WHEN** user writes `lab(50% 20 30)`
- **THEN** the system creates a Color with `ColorSpace::Lab` and channels `[50.0, 20.0, 30.0]`, alpha = 1.0

#### Scenario: lab() with alpha
- **WHEN** user writes `lab(50% 20 30 / 0.5)`
- **THEN** the system creates a Lab color with alpha = 0.5

#### Scenario: lab() with none alpha
- **WHEN** user writes `lab(50% 20 30 / none)`
- **THEN** the system creates a Lab color with alpha = NaN (missing channel)

#### Scenario: lab() serialization roundtrip
- **WHEN** user creates `lab(50% 20 30)` and serializes
- **THEN** the output SHALL be `lab(50% 20 30)` (preserving Lab format)

#### Scenario: lab() missing channels
- **WHEN** user writes `lab(none 20 30)`
- **THEN** the system creates a Lab color with L = NaN

### Requirement: lab L% extraction
The system SHALL interpret `L%` in `lab()` as a percentage value where `50%` maps to `50.0`.

#### Scenario: lab L percentage
- **WHEN** user writes `lab(50% ...)`
- **THEN** the L channel value is 50.0 (not 0.5)
