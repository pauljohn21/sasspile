## ADDED Requirements

### Requirement: ColorSpace enum SHALL represent all CSS Color 4 spaces
The system SHALL define a `ColorSpace` enum with 16 variants (Rgb, Srgb, SrgbLinear, DisplayP3, DisplayP3Linear, A98Rgb, ProphotoRgb, Rec2020, XyzD65, XyzD50, Hsl, Hwb, Lab, Lch, Oklab, Oklch). Each variant SHALL NOT carry channel data—channel values are stored in `Color.channels`.

#### Scenario: ColorSpace from string
- **WHEN** `ColorSpace::from_str("oklch")` is called
- **THEN** it SHALL return `Some(ColorSpace::Oklch)`

#### Scenario: ColorSpace from unknown string
- **WHEN** `ColorSpace::from_str("unknown-space")` is called
- **THEN** it SHALL return `None`

#### Scenario: ColorSpace as string
- **WHEN** `ColorSpace::XyzD50.as_str()` is called
- **THEN** it SHALL return `"xyz-d50"`

#### Scenario: ColorSpace is_legacy
- **WHEN** `ColorSpace::Rgb.is_legacy()` is called
- **THEN** it SHALL return `true`
- **WHEN** `ColorSpace::Oklch.is_legacy()` is called
- **THEN** it SHALL return `false`

### Requirement: ColorOutput enum SHALL control serialization format independently
The system SHALL define a `ColorOutput` enum with variants `Auto`, `RgbExplicit`, and `RgbPercent` to decouple output mode from color space. `Auto` SHALL produce hex or named color output; `RgbExplicit` SHALL produce `rgb()`/`rgba()` output; `RgbPercent` SHALL produce `rgb(r%, g%, b%)` output.

#### Scenario: Auto output produces hex
- **WHEN** a Color with `output: ColorOutput::Auto` and `legacy_rgb: [255, 0, 0]` is serialized
- **THEN** it SHALL output `#ff0000` or the named color `red`

#### Scenario: RgbExplicit output produces rgb()
- **WHEN** a Color with `output: ColorOutput::RgbExplicit` and `legacy_rgb: [255, 0, 0]` is serialized
- **THEN** it SHALL output `rgb(255, 0, 0)`

### Requirement: Color struct SHALL store space, channels, alpha, and output separately
The system SHALL redefine `Color` as a struct with fields `space: ColorSpace`, `channels: [f64; 3]`, `alpha: f64`, `output: ColorOutput`, and `legacy_rgb: [f64; 3]`. The `ColorFormat` enum SHALL be removed.

#### Scenario: Color construction with space
- **WHEN** `Color { space: ColorSpace::Hsl, channels: [0.0, 1.0, 0.5], alpha: 1.0, output: ColorOutput::Auto, legacy_rgb: [255.0, 0.0, 0.0] }` is constructed
- **THEN** it SHALL serialize as `red` (named color lookup from legacy_rgb)

#### Scenario: Color from HSL with RgbPercent output
- **WHEN** a Color has `space: ColorSpace::Hsl`, `channels: [0.0, 1.0, 0.5]`, `output: ColorOutput::RgbPercent`
- **THEN** it SHALL serialize as `rgb(100%, 0%, 0%)`

### Requirement: ChannelSet enum SHALL group channels by color space
The system SHALL define a `ChannelSet` enum with variants `Hsl(HslChannel)`, `Hwb(HwbChannel)`, `Rgb(RgbChannel)`, `Lab(LabChannel)`, `Lch(LchChannel)`, `Oklab(OklabChannel)`, `Oklch(OklchChannel)`, and `Xyz(XyzChannel)`. Each sub-enum SHALL enumerate the valid channel names for that space.

#### Scenario: Channel from string within space
- **WHEN** `ChannelSet::from_str(ColorSpace::Hsl, "hue")` is called
- **THEN** it SHALL return `Some(ChannelSet::Hsl(HslChannel::Hue))`

#### Scenario: Invalid channel for space
- **WHEN** `ChannelSet::from_str(ColorSpace::Hsl, "chroma")` is called
- **THEN** it SHALL return `None` (chroma is not an HSL channel)

### Requirement: Color space conversion SHALL use enum matching not string comparison
All functions that match on color space (is_same_space, convert_space, to_gamut, channel extraction) SHALL use `ColorSpace` enum matching instead of `&str` comparison. The compiler SHALL guarantee exhaustiveness.

#### Scenario: Convert space via enum
- **WHEN** `convert_space(color, ColorSpace::Oklch)` is called with an sRGB color
- **THEN** it SHALL match `ColorSpace::Oklch` and perform sRGB→Oklch conversion
- **AND** no `&str` comparison SHALL be used in the match arms
