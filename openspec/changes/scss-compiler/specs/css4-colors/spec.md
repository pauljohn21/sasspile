## ADDED Requirements

### Requirement: CSS4 Color SHALL support `color()` function with color space
The compiler SHALL parse and evaluate `color(srgb 0.5 0.5 0.5)`, `color(display-p3 1 0 0)`, etc.

#### Scenario: srgb color space
- **WHEN** evaluating `color(srgb 0.5 0.5 0.5)`
- **THEN** result is `Color(128, 128, 128, 1.0)` (approximate)

#### Scenario: display-p3 color space
- **WHEN** evaluating `color(display-p3 1 0 0)`
- **THEN** result is `Color(255, 0, 0, 1.0)` (mapped to srgb output)

#### Scenario: custom color space
- **WHEN** evaluating `color(--my-cspace 0.2 0.4 0.6)`
- **THEN** result preserves custom space identifier with components

### Requirement: CSS4 Color SHALL support `lab()`, `lch()`, `oklab()`, `oklch()`
The compiler SHALL parse and evaluate CSS4 perceptual color functions.

#### Scenario: oklab evaluation
- **WHEN** evaluating `oklab(0.628 0.225 0.126)`
- **THEN** result is approximately the reference color (red-orange)

#### Scenario: oklch with alpha
- **WHEN** evaluating `oklch(60% 0.15 30 / 0.5)`
- **THEN** result has alpha = 0.5 and converted RGB components

#### Scenario: lch lightness-chroma-hue
- **WHEN** evaluating `lch(50% 100 120)`
- **THEN** result converts to srgb Color value

### Requirement: CSS4 Color SHALL support color mixing with `color-mix()`
The compiler SHALL evaluate `color-mix(in srgb, red 50%, blue)`.

#### Scenario: srgb mix 50/50
- **WHEN** evaluating `color-mix(in srgb, red 50%, blue)`
- **THEN** result is `Color(128, 0, 128, 1.0)` (purple)

#### Scenario: oklch mix
- **WHEN** evaluating `color-mix(in oklch, red 30%, blue)`
- **THEN** result uses oklch interpolation for perceptually uniform mixing

#### Scenario: Missing hue handling
- **WHEN** mixing colors with missing hues in achromatic context
- **THEN** compiler adjusts hue replacement per CSS Color 4 spec

### Requirement: CSS4 Color SHALL support relative color syntax
The compiler SHALL parse `from <color>` relative color derivations.

#### Scenario: Hue adjustment via from
- **WHEN** evaluating `oklch(from red calc(l * 1.2) c h)`
- **THEN** result has 20% increased lightness, keeping same chroma and hue

#### Scenario: Alpha reduction via from
- **WHEN** evaluating `rgb(from blue r g b / 0.5)`
- **THEN** result is `Color(0, 0, 255, 0.5)`

### Requirement: CSS4 Color SHALL support `light-dark()` function
The compiler SHALL evaluate `light-dark($light, $dark)`; resolved at computed-value time (or eagerly if both are literals).

#### Scenario: Eager literal resolution
- **WHEN** `light-dark(white, black)` is used without custom props
- **THEN** compiler resolves to `white` (light value) by default in current output context

### Requirement: CSS4 Color SHALL support `color-adjust()` and `hwb()`
The compiler SHALL handle legacy convenience functions for CSS Color 4.

#### Scenario: hwb evaluation
- **WHEN** evaluating `hwb(0 0 0)` (red with no whiteness/blackness)
- **THEN** result is `Color(255, 0, 0, 1.0)`
