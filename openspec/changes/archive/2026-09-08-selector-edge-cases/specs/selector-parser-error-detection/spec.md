## ADDED Requirements

### Requirement: selector parser attribute selector closure check
The selector parser SHALL detect unclosed attribute selectors and raise a parse error.

#### Scenario: Unclosed attribute selector
- **WHEN** parsing `[c` (missing closing `]`)
- **THEN** a parse error SHALL be raised with message indicating unexpected end of input

#### Scenario: Partially closed attribute selector
- **WHEN** parsing `[foo=bar` (missing closing `]`)
- **THEN** a parse error SHALL be raised

#### Scenario: Valid attribute selector passes
- **WHEN** parsing `[foo="bar"]` (complete attribute selector)
- **THEN** parsing SHALL succeed without error

### Requirement: selector append invalid input error
`selector-append` SHALL raise an error when passed invalid selector input that cannot be parsed.

#### Scenario: Append with unclosed attribute selector
- **WHEN** calling `selector.append("[c", "d")`
- **THEN** an error SHALL be raised indicating the selector is invalid

#### Scenario: Append with invalid characters
- **WHEN** calling `selector.append("!invalid", ".b")`
- **THEN** an error SHALL be raised indicating invalid selector token
