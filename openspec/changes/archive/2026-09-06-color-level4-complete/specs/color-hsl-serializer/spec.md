## MODIFIED Requirements

### Requirement: hsl/hsla serialization precision
The system SHALL maintain backward-compatible HSL output format.

**Previous behavior**: HSL values use integer hue, integer saturation/lightness percentages.
**Modified behavior**: No change to core format; precision improvements only affect edge cases.

#### Scenario: Standard HSL output
- **WHEN** user creates `hsl(120, 50%, 50%)` and serializes
- **THEN** the output is `hsl(120, 50%, 50%)`

#### Scenario: HSL hue precision
- **WHEN** user creates HSL from a conversion (hue = 120.5)
- **THEN** the output preserves `120.5deg` format
