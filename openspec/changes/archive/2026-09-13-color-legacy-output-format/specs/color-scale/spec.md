## ADDED Requirements

### Requirement: scale-color SHALL preserve input color space for legacy colors
When `color.scale` is called on a legacy color, it SHALL preserve the input color space.

#### Scenario: scale-color on HSL input
- **WHEN** `color.scale(hsl(30deg 20% 40%), $saturation: 50%)` is called
- **THEN** the output is in HSL format

#### Scenario: scale-color RGB output
- **WHEN** `color.scale(sienna, $red: 12%, $green: 24%, $blue: 48%)` is called
- **THEN** the output is `rgb(67.22%, 48.44%, 57.18%)`
