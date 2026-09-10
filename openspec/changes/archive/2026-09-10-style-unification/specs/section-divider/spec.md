## ADDED Requirements

### Requirement: Section divider SHALL use consistent character
All section dividers SHALL use U+2500 (BOX DRAWINGS LIGHT HORIZONTAL, `─`) as the divider character.

#### Scenario: Replacing existing dividers
- **WHEN** a file contains `═══` (U+2550) dividers (e.g., `reactor_test.rs`)
- **THEN** they SHALL be replaced with `───` (U+2500) format

#### Scenario: Replacing em dash dividers
- **WHEN** a file contains `——` (U+2014) dividers (e.g., `tests/interp_test.rs`)
- **THEN** they SHALL be replaced with `───` (U+2500) format

#### Scenario: Files without dividers
- **WHEN** a file currently has no section dividers but has logical sections
- **THEN** dividers SHALL be added only if doing so improves readability (not mandatory for small files)

### Requirement: Section divider SHALL be right-aligned to 78 columns
Section dividers SHALL be padded with `─` characters to align the right edge at column 78.

#### Scenario: Creating a new divider
- **WHEN** a developer adds a `// ─── Data Structures` divider
- **THEN** the full line SHALL be exactly 78 characters: `// ─── Data Structures ───────────────────────────────────────────`

### Requirement: Divider format SHALL include spaces around name
Section dividers SHALL follow the format: `// ─── Name ───` with exactly one space between the dashes and the section name.

#### Scenario: Divider with short name
- **WHEN** the section name is "Types" (5 chars)
- **THEN** the divider SHALL be `// ─── Types ─────────────────────────────────────────────────`

#### Scenario: Divider with long name
- **WHEN** the section name is "sass-spec Reference" (18 chars)
- **THEN** the divider SHALL be `// ─── sass-spec Reference ───────────────────────`
