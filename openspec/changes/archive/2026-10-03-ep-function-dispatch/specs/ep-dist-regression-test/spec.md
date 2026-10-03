## ADDED Requirements

### Requirement: EP dist comparison test
The system SHALL compare sasspile output against EP official dist output for all entry-point scss files, normalizing CSS before comparison.

#### Scenario: Normalized comparison
- **WHEN** sasspile compiles a scss file and the official dist has a corresponding CSS
- **THEN** the test SHALL normalize both outputs (strip comments, collapse whitespace) before comparing

#### Scenario: Missing dist reference
- **WHEN** a scss file has no corresponding dist CSS (e.g., partial files)
- **THEN** the test SHALL skip comparison for that file and count it as "no reference"

#### Scenario: sasspile compilation failure
- **WHEN** sasspile fails to compile an EP scss file
- **THEN** the test SHALL count it as "sasspile fail" and continue with other files

#### Scenario: Reporting
- **WHEN** the test completes all comparisons
- **THEN** it SHALL emit a summary with counts: identical (100% match), diff (any mismatch), no_dist_ref, sasspile_fail

### Requirement: Acceptance threshold
The system SHALL report PASS only if sasspile generates output that matches official dist (after normalization) for ≥80% of EP entry files.

#### Scenario: 80% threshold met
- **WHEN** ≥97/121 files produce identical normalized output
- **THEN** the test suite reports PASS with success rate

#### Scenario: Below threshold
- **WHEN** <97/121 files produce identical normalized output
- **THEN** the test SHALL list all diffing files for diagnostic purposes
