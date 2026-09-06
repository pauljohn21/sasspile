## ADDED Requirements

### Requirement: modern color serialization precision
The system SHALL serialize modern color space outputs with sufficient precision to match expected values.

#### Scenario: Lab serialization precision
- **WHEN** user creates `lab(75% 20 30)` and serializes
- **THEN** the output SHALL be `lab(75% 20 30)` (no precision loss)

#### Scenario: Oklab float precision
- **WHEN** user creates `oklab(62.5% 0.12 -0.08)` and serializes
- **THEN** the output preserves the decimal values with ~10 significant digits

#### Scenario: Lch hue precision
- **WHEN** user creates `lch(50% 30 180.5deg)` and serializes
- **THEN** the output SHALL preserve the hue value as `180.5deg`

### Requirement: roundtrip stability
The system SHALL maintain roundtrip stability for modern color spaces (serialize then parse yields equivalent values).

#### Scenario: Lab roundtrip
- **WHEN** user creates `lab(50% 20 30)` then serializes and re-parses
- **THEN** the re-parsed values are within 1e-6 of original

#### Scenario: Oklch roundtrip
- **WHEN** user creates `oklch(50% 0.1 180)` then serializes and re-parses
- **THEN** the re-parsed values are within 1e-6 of original
