## ADDED Requirements

### Requirement: oklch() constructor
The system SHALL support the `oklch(L% C Hdeg [/ alpha])` CSS Color 4 constructor that creates a color in OKLCH color space.

#### Scenario: Basic oklch construction
- **WHEN** user writes `oklch(50% 0.1 180deg)`
- **THEN** the system creates a Color with `ColorSpace::Oklch` and channels `[0.5, 0.1, 180.0]`

#### Scenario: oklch() with alpha
- **WHEN** user writes `oklch(50% 0.1 180 / 0.5)`
- **THEN** the system creates an Oklch color with alpha = 0.5

#### Scenario: oklch() serialization
- **WHEN** user creates `oklch(50% 0.1 180deg)` and serializes
- **THEN** the output SHALL be `oklch(50% 0.1 180deg)`

#### Scenario: oklch() hue normalization for chroma=0
- **WHEN** user creates `oklch(50% 0 180)` and serializes
- **THEN** the hue SHALL be output as `none`

### Requirement: oklch L% extraction
The system SHALL normalize `L%` in `oklch()` by dividing by 100.

#### Scenario: oklch L percentage
- **WHEN** user writes `oklch(50% ...)`
- **THEN** the internal L value is 0.5
- **AND** serialized output preserves `50%` format
