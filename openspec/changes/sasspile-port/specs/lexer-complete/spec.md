# Spec Delta

## Purpose

Complete SCSS lexer that transforms source text into a stream of tokens, supporting all SCSS syntax including interpolation, comments, all @-rules, string escapes, numbers with units, and operators. Built as an rxrust Observable wrapping sasspile's scanner algorithm.

## ADDED Requirements

### Requirement: Token stream output
The lexer SHALL produce a `Shared Observable<Token>` from SCSS source text. Each lexeme SHALL emit exactly one Token. The stream SHALL complete with `Token::Eof` at end of input.

#### Scenario: Simple rule lexing
- **WHEN** scanning `a { color: red; }`
- **THEN** produces tokens: Ident("a"), LBrace, Ident("color"), Colon, Ident("red"), Semicolon, RBrace, Eof

#### Scenario: Variable lexing
- **WHEN** scanning `$primary: #3498db;`
- **THEN** produces: Dollar, Ident("primary"), Colon, HashId("3498db"), Semicolon

### Requirement: String tokenization
The lexer SHALL handle single-quoted, double-quoted, and unquoted strings with escape sequences. Interpolated strings SHALL emit Ident/Str fragments with InterpolationStart/InterpolationEnd around `#{}` blocks.

#### Scenario: Double-quoted string
- **WHEN** scanning `"hello world"`
- **THEN** produces: Str("hello world")

#### Scenario: Interpolated string
- **WHEN** scanning `"item-#{$i}"`
- **THEN** produces: Str("item-"), InterpolationStart, Dollar, Ident("i"), InterpolationEnd, Eof

### Requirement: Number with unit
The lexer SHALL recognize numbers with optional decimal points and optional unit suffixes (px, em, rem, %, deg, s, etc.). Scientific notation (1e3) SHALL be supported.

#### Scenario: Number with px unit
- **WHEN** scanning `16px`
- **THEN** produces: Number(16.0, Some("px"))

#### Scenario: Float number
- **WHEN** scanning `1.5`
- **THEN** produces: Number(1.5, None)

### Requirement: Comment handling
The lexer SHALL distinguish single-line comments (`//`) from multi-line comments (`/* */`). Silent comments SHALL be suppressed from output. Preserved comments (`/*! */`) SHALL emit Comment tokens.

#### Scenario: Silent comment
- **WHEN** scanning `// this is silent`
- **THEN** produces: Eof (comment suppressed)

#### Scenario: Preserved comment
- **WHEN** scanning `/*! important */`
- **THEN** produces: Comment("! important ")

### Requirement: @-rule recognition
The lexer SHALL recognize all SCSS @-rules: @use, @forward, @import, @include, @mixin, @function, @return, @if, @else, @for, @each, @while, @extend, @at-root, @content, @warn, @debug, @error, @media, @supports, @keyframes, @font-face, @page, @namespace, @charset, @custom-media, @custom-selector.

#### Scenario: @use rule
- **WHEN** scanning `@use "sass:math";`
- **THEN** produces: AtUse, Str("sass:math"), Semicolon

#### Scenario: @include rule
- **WHEN** scanning `@include clearfix;`
- **THEN** produces: AtInclude, Ident("clearfix"), Semicolon

### Requirement: Operator tokenization
The lexer SHALL recognize all SCSS operators: +, -, *, /, %, ==, !=, <, >, <=, >=, and, or, not, =, :, ;, ,, ., (, ), {, }, [, ], ?, !.

#### Scenario: Comparison operators
- **WHEN** scanning `$x == 1`
- **THEN** produces: Dollar, Ident("x"), Eq, Number(1.0, None)

### Requirement: Selector tokenization
The lexer SHALL recognize selector-specific tokens: class (.), id (#hex or #name), & (parent reference), :: (pseudo-element), : (pseudo-class), >, +, ~ (combinators), [ ] (attribute selectors).

#### Scenario: Complex selector
- **WHEN** scanning `.btn:hover > span`
- **THEN** produces: Dot, Ident("btn"), Colon, Ident("hover"), Gt, Ident("span")
