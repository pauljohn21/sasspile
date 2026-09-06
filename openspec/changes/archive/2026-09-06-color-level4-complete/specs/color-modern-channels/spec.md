## ADDED Requirements

### Requirement: color.channel modern space support
The system SHALL support `color.channel()` for modern color spaces with their canonical channel names.

#### Scenario: Lab channel lookup
- **WHEN** user calls `color.channel(lab(50% 20 30), 'lightness')`
- **THEN** the result is `50`

#### Scenario: Lab a channel
- **WHEN** user calls `color.channel(lab(50% 20 30), 'a')`
- **THEN** the result is `20`

#### Scenario: Lab b channel
- **WHEN** user calls `color.channel(lab(50% 20 30), 'b')`
- **THEN** the result is `30`

#### Scenario: Lch chroma channel
- **WHEN** user calls `color.channel(lch(50% 30 180), 'chroma')`
- **THEN** the result is `30`

#### Scenario: Lch hue channel
- **WHEN** user calls `color.channel(lch(50% 30 180), 'hue')`
- **THEN** the result is `180`

#### Scenario: Oklab channels
- **WHEN** user calls `color.channel(oklab(50% 0.1 -0.2), 'a')`
- **THEN** the result is approximately `0.1`

#### Scenario: Oklch channels
- **WHEN** user calls `color.channel(oklch(50% 0.1 180), 'chroma')`
- **THEN** the result is approximately `0.1`

### Requirement: color.channel missing channel output
The system SHALL output `none` when the requested channel is NaN (missing).

#### Scenario: Lab missing L
- **WHEN** user calls `color.channel(lab(none 20 30), 'lightness')`
- **THEN** the result is `none`
