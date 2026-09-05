## ADDED Requirements

### Requirement: Legacy same-space conversion normalization
The system SHALL normalize legacy colors (RGB, HSL, HWB) during same-space conversion through internal HSL computation, rather than returning the original value unchanged.

#### Scenario: hwb to hwb conversion
- **WHEN** `color.to-space(hwb(0deg 20% 30%), hwb)` is evaluated
- **THEN** the system SHALL compute the color through HSL intermediates and output `hsl(0, 55.5555555556%, 45%)`

#### Scenario: rgb to rgb conversion
- **WHEN** `color.to-space(rgb(255 0 0), rgb)` is evaluated
- **THEN** the color SHALL be normalized through computation, not returned as-is

### Requirement: Modern space same-space conversion passthrough
The system SHALL return modern color spaces (Lab, Lch, Oklab, Oklch, DisplayP3, etc.) unchanged during same-space conversion.

#### Scenario: lab to lab passthrough
- **WHEN** `color.to-space(lab(50% 40 59.5), lab)` is evaluated
- **THEN** the output SHALL be `lab(50% 40 59.5)` unchanged

#### Scenario: oklch to oklch passthrough
- **WHEN** `color.to-space(oklch(0.7 0.1 180), oklch)` is evaluated
- **THEN** the output SHALL be `oklch(70% 0.1 180deg)` unchanged

### Requirement: Conversion precision alignment
The system SHALL use f64 precision throughout the conversion chain, aligning matrix coefficients with CSS Color 4 specification reference values.

#### Scenario: hwb to srgb conversion
- **WHEN** converting `hwb(0deg 20% 30%)` to sRGB
- **THEN** the sRGB values SHALL be computed with f64 precision using the CSS Color 4 HWB→RGB algorithm

#### Scenario: oklch to lab conversion
- **WHEN** converting oklch to lab
- **THEN** the conversion SHALL go through sRGB intermediates with full f64 precision
