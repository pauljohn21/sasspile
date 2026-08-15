## ADDED Requirements

### Requirement: Diagnostic System SHALL emit structured errors
Every diagnostic SHALL carry: `level` (Error/Warn/Info), `code`, `message`, `location` (SourceSpan), optional `notes`.

#### Scenario: Error with location
- **WHEN** emitting a parse error
- **THEN** `Diagnostic::error("E001", "Unexpected character").with_location(span)` produces a structured diagnostic

#### Scenario: Warning with note
- **WHEN` emitting a deprecation warning
- **THEN** `Diagnostic::warn("W003", "@import is deprecated").with_note("Use @use")` preserves note

### Requirement: Diagnostic System SHALL render human-readable output
Diagnostics SHALL render with source snippet, underline, and color when stderr is a terminal.

#### Scenario: Single-line span
- **WHEN** rendering an error spanning one source line
- **THEN** output shows the source line followed by carets under the problematic span

#### Scenario: Multi-line span
- **WHEN** rendering an error spanning multiple source lines
- **THEN** output shows `start_line..end_line` with snippet of starting line

### Requirement: Diagnostic System SHALL collect multiple errors without early exit
Evaluation SHALL accumulate errors and continue parsing/evaluating when useful (best-effort).

#### Scenario: First error does not block reporting others
- **WHEN** source has 3 parse errors
- **THEN** all 3 are reported if the parser can recover, not just the first

#### Scenario: Error threshold exceeded
- **WHEN** errors exceed `max_errors` (default 100)
- **THEN** compilation halts with summary of suppressed errors count

### Requirement: Diagnostic System SHALL map to source spans
The compiler SHALL track SourceSpan (start..end offsets) through every stage (lex -> parse -> eval).

#### Scenario: Eval error maps to source
- **WHEN** a Sass error occurs during evaluation at a specific expression
- **THEN** the resulting Diagnostic points to the original source expression span, not the expanded form
