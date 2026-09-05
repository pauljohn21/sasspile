## ADDED Requirements

### Requirement: color.mix supports modern color spaces
The system SHALL support mixing colors in modern color spaces (lab, lch, oklab, oklch, display-p3, srgb, etc.) by performing linear interpolation in the source color's space.

#### Scenario: mix two colors in oklch space
- **WHEN** mixing two oklch colors with `color.mix(oklch(0.7 0.1 180), oklch(0.5 0.2 120))`
- **THEN** the result SHALL be an oklch color with channels interpolated linearly

#### Scenario: mix with weight parameter
- **WHEN** mixing with `color.mix($color1, $color2, $weight: 30%)`
- **THEN** 30% of color1 and 70% of color2 SHALL be used in the interpolation

#### Scenario: mix legacy colors stays legacy
- **WHEN** mixing two legacy RGB colors (e.g., `red` and `blue`)
- **THEN** the result SHALL be a legacy RGB color output as hex or named color

### Requirement: color.mix preserves color space
The system SHALL preserve the color space of the first color argument when mixing.

#### Scenario: mix srgb color with display-p3 color
- **WHEN** mixing `color(srgb 0.5 0.2 0.8)` with `color(display-p3 0.3 0.6 0.1)`
- **THEN** the result SHALL be in srgb space, with the second color converted to srgb before mixing

#### Scenario: mix legacy with modern
- **WHEN** mixing a legacy color (`red`) with a modern color (`oklch(0.7 0.1 180)`)
- **THEN** the legacy color SHALL be converted to the modern space before mixing
