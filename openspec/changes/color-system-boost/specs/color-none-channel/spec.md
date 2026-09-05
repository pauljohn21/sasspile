## ADDED Requirements

### Requirement: Parse none keyword in color functions
The system SHALL parse the `none` keyword as a missing channel value in all CSS Color 4 color functions (`hsl()`, `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()`, `color()`).

#### Scenario: none in hwb hue channel
- **WHEN** parsing `hwb(none 20% 30%)`
- **THEN** the hue channel SHALL be marked as missing (NaN)

#### Scenario: none in lab lightness
- **WHEN** parsing `lab(none 40 59.5)`
- **THEN** the lightness channel SHALL be marked as missing (NaN)

#### Scenario: none in color() channels
- **WHEN** parsing `color(srgb none 0.5 0.7)`
- **THEN** the red channel SHALL be marked as missing (NaN)

### Requirement: Serialize none for missing channels
The system SHALL serialize missing channels (NaN) as the literal `none` keyword in color output.

#### Scenario: hwb with missing hue
- **WHEN** serializing a color with NaN hue in HWB space
- **THEN** the output SHALL be `hwb(none W% B%)`

#### Scenario: oklch with missing hue
- **WHEN** serializing an oklch color with chroma 0 (hue is powerless)
- **THEN** the hue component SHALL be `none`

### Requirement: Missing channels treated as zero in conversions
The system SHALL treat missing channels (NaN) as 0.0 when performing color space conversions.

#### Scenario: hwb with none hue converts to hsl
- **WHEN** converting `hwb(none 20% 30%)` to HSL
- **THEN** the hue SHALL be treated as 0 during conversion, producing `hsl(0, ...)`

#### Scenario: none in conversion to different space
- **WHEN** converting a color with a missing channel to a different color space
- **THEN** the missing channel SHALL be substituted with 0.0 before mathematical conversion
