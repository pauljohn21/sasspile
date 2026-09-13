## ADDED Requirements

### Requirement: change-color SHALL preserve input color space for legacy colors
When `color.change` is called on a legacy color, it SHALL preserve the input color space.

#### Scenario: change-color on HSL input
- **WHEN** `color.change(hsl(30deg 20% 40%), $hue: 180)` is called
- **THEN** the output is in HSL format

#### Scenario: change-color RGB channels
- **WHEN** `color.change(black, $green: -50)` is called
- **THEN** the output uses appropriate format (HSL for out-of-range)
