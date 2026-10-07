# Spec Delta

## Purpose

Complete SCSS parser that transforms a Token stream into an AstNode stream, covering all SCSS syntax: variables, rules, @-rules (use/forward/import/include/mixin/function/if/for/each/while/extend/at-root/content/error), expressions with Pratt parsing, selectors with interpolation, and nested structures. Built as an rxrust Observable.

## ADDED Requirements

### Requirement: Expression parsing (Pratt algorithm)
The parser SHALL implement Pratt (precedence climbing) expression parsing with correct precedence: or < and < comparison < addition/subtraction < multiplication/division/modulo < unary < primary.

#### Scenario: Operator precedence
- **WHEN** parsing `1 + 2 * 3`
- **THEN** produces BinOp(Add, 1, BinOp(Mul, 2, 3))

#### Scenario: Parenthesized grouping
- **WHEN** parsing `(1 + 2) * 3`
- **THEN** produces BinOp(Mul, BinOp(Add, 1, 2), 3)

#### Scenario: Unary negation
- **WHEN** parsing `-$x`
- **THEN** produces UnaryOp(Neg, VariableRef("x"))

### Requirement: Variable declaration parsing
The parser SHALL recognize `$name: value;` with optional flags `!default` and `!global`. Variable names SHALL support both kebab-case and camelCase (normalized to underscore).

#### Scenario: Default flag
- **WHEN** parsing `$color: red !default;`
- **THEN** produces VariableDecl { name: "color", value: Literal(Color(...)), flags: VarFlags { default: true, global: false } }

#### Scenario: Global flag
- **WHEN** parsing `$theme: dark !global;`
- **THEN** produces VariableDecl with global=true

### Requirement: Rule parsing
The parser SHALL recognize `selector { body }` blocks. Selectors SHALL support interpolation `#{}`. Nested rules SHALL be parsed recursively.

#### Scenario: Nested rule
- **WHEN** parsing `.parent { .child { color: red; } }`
- **THEN** produces Rule { selector: ".parent", body: [Rule { selector: ".child", body: [Decl("color", red)] }] }

### Requirement: @-rule parsing
The parser SHALL parse all @-rules with their specific syntax: @use (with 'as', 'with'), @forward (with 'show', 'hide', 'as prefix-*'), @import, @include (with args, @content), @mixin (with params), @function (with params), @return, @if/@else if/@else, @for, @each (multi-var), @while, @extend, @at-root, @warn, @debug, @error.

#### Scenario: @for loop
- **WHEN** parsing `@for $i from 1 through 3 { .col-#{$i} { width: #{$i * 33.33}%; } }`
- **THEN** produces For { var: "i", from: 1, to: 3, inclusive: true, body: [...] }

#### Scenario: @each multi-var
- **WHEN** parsing `@each $key, $val in $map { ... }`
- **THEN** produces Each { vars: ["key", "val"], list: VariableRef("map"), body: [...] }

#### Scenario: @mixin with params
- **WHEN** parsing `@mixin btn($bg: blue, $fg: white) { background: $bg; color: $fg; }`
- **THEN** produces MixinDef { name: "btn", params: [Param("bg", blue), Param("fg", white)], body: [...] }

### Requirement: Selector interpolation
The parser SHALL handle `#{}` interpolation within selectors. The interpolation SHALL produce interpolated selector strings that are resolved at eval time.

#### Scenario: Interpolated class
- **WHEN** parsing `.col-#{$i} { ... }`
- **THEN** produces Rule with selector containing Interpolation nodes

### Requirement: Value expression parsing
The parser SHALL recognize all Sass value literals: numbers (with units), strings (with interpolation), colors (named/hex/function), lists (comma/space separated, bracketed), maps (key: value pairs), function calls (positional + keyword args), `null`, `true`, `false`.

#### Scenario: Map literal
- **WHEN** parsing `(primary: blue, danger: red)`
- **THEN** produces MapLiteral with entries [(primary, blue), (danger, red)]

#### Scenario: Function call with kwargs
- **WHEN** parsing `rgba($color, $alpha: 0.5)`
- **THEN** produces FunctionCall { name: "rgba", args: [Positional($color), Keyword("alpha", 0.5)] }

### Requirement: Comment parsing
The parser SHALL parse single-line and multi-line comments. Silent comments (`//`) SHALL produce no AST node. Multi-line comments SHALL produce Comment nodes when preserved.

#### Scenario: Silent comment
- **WHEN** parsing `// this comment is silent`
- **THEN** produces no AST node
