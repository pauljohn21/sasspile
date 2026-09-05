## ADDED Requirements

### Requirement: color() single-argument passthrough
The system SHALL pass through a Color value when `color()` is called with a single `Value::Color` argument.

#### Scenario: color wraps another color
- **WHEN** evaluating `color(color(srgb 0.2 0.5 0.7))`
- **THEN** the result SHALL be the inner color value unchanged

#### Scenario: color with existing oklch color
- **WHEN** evaluating `color(oklch(0.7 0.1 180))`
- **THEN** the result SHALL be an oklch color

### Requirement: Each color operation has independent implementation per space
The system SHALL implement each color operation function (adjust, change, scale) as a fully independent function per color space. No two color spaces SHALL share mutable state or branching logic within a single function body.

#### Scenario: adjust oklch independent
- **WHEN** calling `color.adjust()` on an oklch color
- **THEN** the oklch-specific adjust function SHALL be called, with no shared logic from hsl or lab implementations

#### Scenario: change lab independent
- **WHEN** calling `color.change()` on a lab color
- **THEN** the lab-specific change function SHALL be called independently

#### Scenario: scale hsl independent
- **WHEN** calling `color.scale()` on an hsl color
- **THEN** the hsl-specific scale function SHALL be called independently

### Requirement: color.invert supports modern spaces
The system SHALL support `color.invert()` for modern color spaces by converting to sRGB, inverting, and converting back.

#### Scenario: invert oklch color
- **WHEN** inverting `oklch(0.7 0.1 180)`
- **THEN** the hue SHALL be rotated 180 degrees within the oklch space

#### Scenario: invert legacy color
- **WHEN** inverting `red`
- **THEN** the result SHALL be the complement color (legacy HSL rotation)

### Requirement: color.grayscale supports modern spaces
The system SHALL support `color.grayscale()` for modern color spaces by setting chroma to 0 in cylindrical spaces or converting to luminance in rectangular spaces.

#### Scenario: grayscale oklch
- **WHEN** applying grayscale to `oklch(0.7 0.1 180)`
- **THEN** the chroma SHALL become 0, producing `oklch(70% 0 none)`
