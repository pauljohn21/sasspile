## MODIFIED Requirements

### Requirement: rgb/rgba serialization precision
The system SHALL maintain backward-compatible rgb output while supporting higher precision when needed.

**Previous behavior**: RGB values are rounded to integers (0-255).
**Modified behavior**: RGB values continue to round to integers; no change to core behavior.

#### Scenario: Standard RGB output
- **WHEN** user creates `rgb(255, 0, 0)` and serializes
- **THEN** the output is `red` (named) or `rgb(255, 0, 0)`

#### Scenario: rgba alpha precision
- **WHEN** user creates `rgba(255, 0, 0, 0.5)` and serializes
- **THEN** the output is `rgba(255, 0, 0, 0.5)`
