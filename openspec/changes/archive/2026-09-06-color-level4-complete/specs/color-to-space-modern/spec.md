## ADDED Requirements

### Requirement: color.to-space modern space conversion
The system SHALL support `color.to-space()` for converting colors between modern color spaces.

#### Scenario: Lab to sRGB
- **WHEN** user calls `color.to-space(lab(50% 20 30), srgb)`
- **THEN** the result is an sRGB color (channels in 0-1 range)

#### Scenario: sRGB to Lab
- **WHEN** user calls `color.to-space(red, lab)`
- **THEN** the result is a Lab color

#### Scenario: Oklab to sRGB
- **WHEN** user calls `color.to-space(oklab(50% 0.1 -0.2), srgb)`
- **THEN** the result is an sRGB color

#### Scenario: sRGB to Oklch
- **WHEN** user calls `color.to-space(rgb(255, 0, 0), oklch)`
- **THEN** the result is an Oklch color

#### Scenario: Lab to Lch
- **WHEN** user calls `color.to-space(lab(50% 20 30), lch)`
- **THEN** the result is an Lch color (polar transform of Lab)

#### Scenario: Lch to Oklch
- **WHEN** user calls `color.to-space(lch(50% 30 180), oklch)`
- **THEN** the result is an Oklch color (via Lab->XYZ->sRGB->Oklab->Oklch path)
