## ADDED Requirements

### Requirement: invert SHALL preserve input color space for legacy colors
When `color.invert` is called on a legacy color (RGB/HSL/HWB), it SHALL preserve the input color space in the output rather than always converting to RGB.

#### Scenario: invert HSL color outputs HSL
- **WHEN** `color.invert(hsl(30deg 20% 40%))` is called
- **THEN** the output is `hsl(210, 20%, 60%)`

#### Scenario: invert RGB color outputs hex or rgb
- **WHEN** `color.invert(#ff0000)` is called
- **THEN** the output is `#00ffff` (or equivalent cyan)

#### Scenario: invert HWB color outputs HWB/HSL
- **WHEN** `color.invert(hwb(30deg 20% 30%))` is called
- **THEN** the output preserves HWB space

### Requirement: invert weighted SHALL output rgb percent format
When `color.invert` is called with a `$weight` argument, it SHALL output in `rgb(R%, G%, B%)` format.

#### Scenario: weighted invert on turquoise
- **WHEN** `color.invert(turquoise, 92%)` is called
- **THEN** the output is `rgb(70.92%, 18.21%, 23.48%)`

#### Scenario: weighted invert with low weight
- **WHEN** `color.invert(turquoise, 23%)` is called
- **THEN** the output uses `rgb(R%, G%, B%)` format
