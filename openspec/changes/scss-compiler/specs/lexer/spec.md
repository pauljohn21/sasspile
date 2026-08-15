## ADDED Requirements

### Requirement: Lexer SHALL tokenize SCSS source into Token stream
The lexer SHALL accept SCSS text input and produce a strict sequence of tokens representing identifiers, numbers, strings, operators, and special characters.

#### Scenario: Basic SCSS tokenization
- **WHEN** lexer processes `a { color: red; }`
- **THEN** it produces tokens: `Ident("a")`, `Char('{')`, `Ident("color")`, `Char(':')`, `Ident("red")`, `Char(';')`, `Char('}')`

#### Scenario: Interpolation syntax
- **WHEN** lexer processes `#{$var + 1}px`
- **THEN** it produces tokens: `InterpolationStart`, `Variable("$var")`, `Plus`, `Number(1)`, `InterpolationEnd`, `Ident("px")`

#### Scenario: Number with units
- **WHEN** lexer processes `16px` or `1.5rem`
- **THEN** it produces single token `Number(16, Some(Unit::Px))` or `Number(1.5, Some(Unit::Rem))`

### Requirement: Lexer SHALL support Sass indented syntax
The lexer SHALL accept the indented `.sass` syntax (no braces, no semicolons, indentation-based nesting).

#### Scenario: Indented syntax tokenization
- **WHEN** lexer processes Sass indented syntax with property indentation
- **THEN** it produces appropriate tokens with `Indent`/`Dedent` markers

### Requirement: Lexer SHALL track source positions
Each token SHALL carry its `SourcePosition` (line, column, offset) for error reporting.

#### Scenario: Error location tracking
- **WHEN** lexer encounters invalid syntax at line 5 column 10
- **THEN** all subsequent tokens retain correct line/column information
