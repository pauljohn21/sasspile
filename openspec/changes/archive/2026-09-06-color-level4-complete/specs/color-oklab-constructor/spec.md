## ADDED Requirements

### Requirement: oklab() constructor
The system SHALL support the `oklab(L% a b [/ alpha])` CSS Color 4 constructor that creates a color in OKLab color space.

#### Scenario: Basic oklab construction
- **WHEN** user writes `oklab(50% 0.1 -0.2)`
- **THEN** the system creates a Color with `ColorSpace::Oklab` and channels `[0.5, 0.1, -0.2]` (L% divided by 100)

#### Scenario: oklab() with alpha
- **WHEN** user writes `oklab(50% 0.1 -0.2 / 0.8)`
- **THEN** the system creates an Oklab color with alpha = 0.8

#### Scenario: oklab() serialization
- **WHEN** user creates `oklab(50% 0.1 -0.2)` and serializes
- **THEN** the output SHALL be `oklab(50% 0.1 -0.2)` (preserving Oklab format)

#### Scenario: oklab() with none values
- **WHEN** user writes `oklab(none 0.1 -0.2)`
- **THEN** the system creates an Oklab color with L = NaN

### Requirement: oklab L% normalization
The system SHALL normalize `L%` in `oklab()` by dividing by 100 (OKLab uses 0-1 range internally).

#### Scenario: oklab L percentage conversion
- **WHEN** user writes `oklab(50% ...)`
- **THEN** the internal L value is 0.5 (50% / 100)
- **AND** serialized output preserves `50%` format

#### Scenario: oklab L fraction
- **WHEN** user writes `oklab(0.5 ...)`
- **THEN** the internal L value is 0.5
