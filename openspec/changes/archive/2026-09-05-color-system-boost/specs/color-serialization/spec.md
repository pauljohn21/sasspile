## ADDED Requirements

### Requirement: Color serialization precision
The system SHALL serialize color channel values with up to 10 decimal places, stripping trailing zeros.

#### Scenario: precise oklab output
- **WHEN** serializing `oklab(0.593685 0.107989 0.114769)`
- **THEN** the output SHALL preserve all significant digits without truncation

#### Scenario: trailing zero stripping
- **WHEN** serializing a channel value of 0.5000000000
- **THEN** the output SHALL be `0.5` not `0.5000000000`

### Requirement: Legacy color normalization in output
The system SHALL output legacy colors (RGB, HSL, HWB) using HSL format when the color was created or converted from HWB space.

#### Scenario: hwb created color outputs as hsl
- **WHEN** serializing a color created via `hwb(0deg 20% 30%)` with Auto output
- **THEN** the output SHALL be `hsl(0, 55.5555555556%, 45%)` format, not `hwb(...)` format

#### Scenario: to-space hwb to hwb outputs hsl
- **WHEN** `color.to-space(hwb(0deg 20% 30%), hwb)` is evaluated
- **THEN** the output SHALL be `hsl(0, 55.5555555556%, 45%)`

### Requirement: Modern color space serialization
The system SHALL serialize modern color spaces (Lab, Lch, Oklab, Oklch, DisplayP3, sRGB, XYZ, etc.) using `color()` function syntax with space-specific channel formatting.

#### Scenario: display-p3 serialization
- **WHEN** serializing a DisplayP3 color
- **THEN** the output SHALL be `color(display-p3 R G B)` format

#### Scenario: oklch with deg hue
- **WHEN** serializing an oklch color with hue 180
- **THEN** the output SHALL be `oklch(L% C 180deg)` with deg suffix
