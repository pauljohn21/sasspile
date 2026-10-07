# Spec Delta

## Purpose

Complete SCSS evaluator that transforms AstNode streams into CSS statement streams using the existing rxrust reactive framework. Evaluates variables, rules, control flow, mixins, functions, selector composition, and all @-rules with proper scope management. The evaluation algorithm content is ported from sasspile while retaining the CompilerBus state management and Observable pipeline infrastructure from rx-scss.

## ADDED Requirements

### Requirement: CompilerBus-driven evaluation
The evaluator SHALL use the existing CompilerBus (from rx-scss framework) for state sharing during the evaluation phase. The bus provides channels for variable bindings, mixin/function registry, and scope events. Evaluation logic is stage-internal; state communication is bus-mediated.

#### Scenario: Variable binding via bus
- **WHEN** processing a VariableDecl node
- **THEN** the evaluator publishes the binding through CompilerBus's var_events channel
- **AND** subsequent variable references resolve through the bus subscription chain

### Requirement: Variable scope chain
The evaluator SHALL maintain a scope chain where child scopes inherit parent bindings. Variable lookup SHALL traverse from innermost scope outward. Write operations SHALL only affect the current scope. follows SCSS semantics: @if/@for/@each do NOT create new scopes (only rules and mixins/functions do).

#### Scenario: Nested scope variable access
- **WHEN** declaring `$x: 1` in outer scope, then referencing `$x` in inner scope (rule body)
- **THEN** inner scope reads the value 1 from parent scope

#### Scenario: Scope shadowing
- **WHEN** declaring `$x: 1` in outer scope and `$x: 2` in inner scope
- **THEN** inner scope reads 2, outer scope still reads 1 after inner scope exits

#### Scenario: Flow control does NOT create scope
- **WHEN** declaring `$i: 10` inside `@for $i from 1 through 3`
- **THEN** after the loop, `$i` is still the outer value (or undefined) — the loop variable does not leak

### Requirement: @if evaluation
The evaluator SHALL evaluate conditions with Sass truthiness rules (false and null are falsy, everything else truthy). SHALL support `@if`, `@else if`, `@else` chains.

#### Scenario: True condition
- **WHEN** evaluating `@if $visible { display: block; }` where `$visible: true`
- **THEN** produces CssNode::Decl("display", "block")

#### Scenario: False with else
- **WHEN** evaluating `@if $x { a } @else { b }` where `$x: null`
- **THEN** produces b (else branch)

### Requirement: @for evaluation
The evaluator SHALL evaluate `@for $var from <start> to/through <end> { body }`. `through` is inclusive (≤), `to` is exclusive (<). The loop variable SHALL be bound for each iteration.

#### Scenario: Through inclusive
- **WHEN** evaluating `@for $i from 1 through 3 { ... }`
- **THEN** iterates with $i = 1, 2, 3

#### Scenario: To exclusive
- **WHEN** evaluating `@for $i from 1 to 3 { ... }`
- **THEN** iterates with $i = 1, 2

### Requirement: @each evaluation
The evaluator SHALL evaluate `@each $var in $list` and `@each $k, $v in $map`. Lists iterate by item; maps iterate by key-value pair. Multi-variable destructuring SHALL be supported.

#### Scenario: List iteration
- **WHEN** evaluating `@each $color in red, green, blue { .#{$color} { color: $color; } }`
- **THEN** produces three rules with each color

#### Scenario: Map iteration
- **WHEN** evaluating `@each $key, $val in (a: 1, b: 2) { ... }`
- **THEN** iterates twice: ($key=a, $val=1) and ($key=b, $val=2)

### Requirement: @while evaluation
The evaluator SHALL evaluate `@while $cond { body }` with iteration safety limit (max 10,000 iterations to prevent infinite loops).

#### Scenario: Converging loop
- **WHEN** evaluating `@while $i > 0 { ...; $i: $i - 1; }` starting at $i=5
- **THEN** iterates 5 times then exits

### Requirement: Mixin @include evaluation
The evaluator SHALL resolve mixin definitions, bind arguments (with defaults and rest), create a new scope, and evaluate the mixin body. `@content` blocks SHALL be passed from the caller context.

#### Scenario: Basic include
- **WHEN** evaluating `@include clearfix` where `@mixin clearfix { &::after { content: ""; } }`
- **THEN** produces the clearfix CSS rules

#### Scenario: Parameterized include
- **WHEN** evaluating `@include btn(blue, white)` for `@mixin btn($bg, $fg) { background: $bg; color: $fg; }`
- **THEN** produces background: blue; color: white;

### Requirement: @function evaluation
The evaluator SHALL resolve function definitions, bind arguments, evaluate body until `@return`, and return the value. Functions SHALL have their own scope.

#### Scenario: Function with return
- **WHEN** calling `double(5)` where `@function double($x) { @return $x * 2; }`
- **THEN** returns Value::Number(10.0, None)

### Requirement: Selector composition with &
The evaluator SHALL resolve `&` (parent selector reference) in nested rules. When a child rule's selector contains `&`, it SHALL be replaced with the parent selector. Without `&`, descendant combinator SHALL be applied.

#### Scenario: Ampersand replacement
- **WHEN** child selector is `&:hover` and parent is `.btn`
- **THEN** result is `.btn:hover`

#### Scenario: Descendant combinator
- **WHEN** child selector is `.active` (no &) and parent is `.btn`
- **THEN** result is `.btn .active"

### Requirement: @at-root evaluation
The evaluator SHALL evaluate `@at-root` by hoisting child rules to the document root level, optionally stripping media/supports context.

#### Scenario: Basic at-root
- **WHEN** evaluating `.parent { @at-root .child { color: red; } }`
- **THEN** `.child` rule appears at root level, not nested under `.parent`

### Requirement: @media and @supports evaluation
The evaluator SHALL evaluate nested `@media` and `@supports` rules, preserving the query and nesting the body content.

#### Scenario: @media nesting
- **WHEN** evaluating `@media (min-width: 768px) { .container { width: 750px; } }`
- **THEN** produces Media { query: "(min-width: 768px)", inner: [Rule("container", [Decl("width", "750px")])] }

### Requirement: Builtin function dispatch
The evaluator SHALL dispatch builtin function calls through a registry covering math, string, list, map, color, selector, and meta modules. Function resolution order: user-defined → builtin → css function passthrough.

#### Scenario: Math function call
- **WHEN** calling `math.round(3.7)`
- **THEN** returns Value::Number(4.0, None)

#### Scenario: Color function call
- **WHEN** calling `darken(#ff0000, 20%)`
- **THEN** returns a darker red color value

### Requirement: Error handling during evaluation
The evaluator SHALL produce descriptive errors for: undefined variables, undefined mixins, undefined functions, type mismatches, unit errors, and division by zero.

#### Scenario: Undefined variable
- **WHEN** referencing `$undefined_var`
- **THEN** produces Eval error "Undefined variable: $undefined_var"

#### Scenario: Type mismatch
- **WHEN** calling `math.abs("not a number")`
- **THEN** produces Type error with expected type and actual type
