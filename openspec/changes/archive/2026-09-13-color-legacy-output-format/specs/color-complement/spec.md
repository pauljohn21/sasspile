## ADDED Requirements

### Requirement: complement SHALL preserve input color space
When `color.complement` is called, it SHALL output in the same color space as the input.

#### Scenario: complement HSL color outputs HSL
- **WHEN** `color.complement(hsl(30deg 20% 40%))` is called
- **THEN** the output is in HSL format

#### Scenario: complement RGB color outputs rgb percent
- **WHEN** `color.complement(rgba(turquoise, 0.7))` is called
- **THEN** the output is `rgba(87.84%, 25.10%, 31.37%, 0.7)`
