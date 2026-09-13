## ADDED Requirements

### Requirement: grayscale SHALL preserve input color space
When `color.grayscale` is called on a legacy color, it SHALL preserve the input color space.

#### Scenario: grayscale on HSL color
- **WHEN** `color.grayscale(hsl(30deg 20% 40%))` is called
- **THEN** the output is in HSL format

#### Scenario: grayscale on RGB color
- **WHEN** `color.grayscale(#ff0000)` is called
- **THEN** the output is in hex/rgb format (RGB space preserved)
