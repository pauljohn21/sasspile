## ADDED Requirements

### Requirement: adjust-color HSL channels on RGB input SHALL output rgb percent
When `color.adjust` modifies HSL channels (hue/saturation/lightness) on a non-HSL input, it SHALL output in `rgb(R%, G%, B%)` format.

#### Scenario: adjust black with HSL channels
- **WHEN** `color.adjust(black, $hue: 12, $saturation: 24%, $lightness: 48%)` is called
- **THEN** the output is `rgb(59.52%, 41.088%, 36.48%)`

#### Scenario: adjust red with hue only
- **WHEN** `color.adjust(red, $hue: 123)` is called
- **THEN** the output is `rgb(0%, 100%, 5%)`

### Requirement: adjust-color SHALL preserve HSL space for HSL input
When `color.adjust` is called on an HSL color, it SHALL preserve the HSL color space.

#### Scenario: adjust HSL color hue
- **WHEN** `color.adjust(hsl(30deg 20% 40%), $hue: 180)` is called
- **THEN** the output is in HSL format
