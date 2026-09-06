## ADDED Requirements

### Requirement: lch() constructor
The system SHALL support the `lch(L% C Hdeg [/ alpha])` CSS Color 4 constructor that creates a color in CIE LCH color space.

#### Scenario: Basic lch construction
- **WHEN** user writes `lch(50% 30 180deg)`
- **THEN** the system creates a Color with `ColorSpace::Lch` and channels `[50.0, 30.0, 180.0]`

#### Scenario: lch() with alpha
- **WHEN** user writes `lch(50% 30 180 / 0.5)`
- **THEN** the system creates an Lch color with alpha = 0.5

#### Scenario: lch() serialization
- **WHEN** user creates `lch(50% 30 180deg)` and serializes
- **THEN** the output SHALL be `lch(50% 30 180deg)`

#### Scenario: lch() hue without deg unit
- **WHEN** user writes `lch(50% 30 180)`
- **THEN** the hue value is 180 (degrees implied)

#### Scenario: lch() with none hue when chroma=0
- **WHEN** user creates `lch(50% 0 180)` and serializes
- **THEN** the hue SHALL be output as `none` (CSS Color 4 spec: hue is powerless when chroma=0)

### Requirement: lch C and H extraction
The system SHALL interpret `C` (chroma) as a plain number and `Hdeg` as degrees.

#### Scenario: lch chroma as number
- **WHEN** user writes `lch(50% 30 ...)`
- **THEN** the chroma value is 30.0

#### Scenario: lch hue degrees
- **WHEN** user writes `lch(50% 30 180deg)`
- **THEN** the hue is 180.0 degrees
