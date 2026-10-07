# Spec Delta

## Purpose

Reactive compilation pipeline architecture that retains the existing rxrust Observable framework (CompilerBus + scan operators + Shared::create/from_iter) while filling in sasspile's algorithm content for each stage. The framework is already validated — this specification defines how sasspile's algorithms are integrated into the existing reactive infrastructure without architectural overhaul.

## ADDED Requirements

### Requirement: Retain CompilerBus as evaluation state hub
The reactive pipeline SHALL continue using `CompilerBus` (with its SharedSubject channels for vars/modules/scopes) as the evaluation-phase state management mechanism. The bus provides pub-sub style state sharing between pipeline stages — this is the validated foundation that will NOT be replaced.

#### Scenario: Variable registration via CompilerBus
- **WHEN** the evaluator processes a variable declaration node
- **THEN** the variable binding is published through CompilerBus's var_events channel
- **AND** downstream stages (Serializer) receive the binding via subscription

#### Scenario: Mixin registration via CompilerBus
- **WHEN** the evaluator encounters a @mixin definition
- **THEN** the mixin is registered in CompilerBus's mixin registry
- **AND** subsequent @include calls resolve the mixin through the bus

### Requirement: Observable pipe pattern for stage composition
Each pipeline stage SHALL be an `ObservablePipe` extension trait method that consumes the reactor's current state and produces the next stage's input. Stages are composed via method chaining: `.lex()?.parse()?.evaluate()?.serialize()`.

#### Scenario: Stage chaining
- **WHEN** calling `Reactor::new(input).lex()?.parse()?.evaluate()?.serialize(style)`
- **THEN** each stageObservable consumes the previous output and transforms it to the next representation
- **AND** errors from any stage propagate through the Result type

### Requirement: scan operator for stateful transformations within stages
Individual stages that require stateful processing (e.g., Parser's token-to-AST conversion) SHALL use `scan()` to thread stage-local state through the Observable stream. This is stage-internal state, NOT a replacement for CompilerBus.

#### Scenario: Parser accumulator state
- **WHEN** the parser stage processes a token stream via scan
- **THEN** the scan operator maintains a stage-local accumulator (e.g., current nesting level, interpolation depth)
- **AND** each token emission can update the accumulator for subsequent tokens

### Requirement: Content-only integration — no framework replacement
The sasspile port SHALL NOT replace the existing rxrust framework. Instead, sasspile's algorithms are integrated as:
- Lexer state machine logic → `src/lexer/state.rs`
- Pratt expression parser → `src/parser/expr.rs`
- Evaluator dispatch (variables/rules/@-rules) → `src/eval/mod.rs`
- Serializer formatting → `src/serialize/mod.rs`

#### Scenario: Algorithm port without framework change
- **WHEN** porting sasspile's number-with-unit lexing algorithm
- **THEN** the algorithm is placed in `src/lexer/state.rs` as an ObservablePipe implementation
- **AND** the Observable/Shared/scan infrastructure remains unchanged

### Requirement: Incremental feature enablement
The pipeline SHALL support incremental feature addition. Early phases focus on Bootstrap-required features (variables, nesting, @if/@for/@each, mixins, color functions). Advanced features (@use, @forward, @extend, full color spaces) are deferred to Phase 2.

#### Scenario: Bootstrap Phase 1 feature set
- **WHEN** compiling `bootstrap/scss/bootstrap.scss` in Phase 1
- **THEN** the compiler handles: variables, nested rules, @if/@else, @for/@each/@while, @include/@mixin, @import (legacy), @media nesting, color functions, math functions
- **AND** @use/@forward are NOT required for Bootstrap compilation
