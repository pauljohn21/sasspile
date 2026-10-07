# Spec Delta

## Purpose

End-to-end compilation pipeline that connects all four stages (Lexer, Parser, Evaluator, Serializer) via rxrust Observable streams. Retains the existing `Reactor<State>` type state machine framework and CompilerBus state management, upgrading only the algorithm content within each stage from basic rx-scss skeleton to sasspile-level implementations.

## ADDED Requirements

### Requirement: Four-stage Observable pipeline
The pipeline SHALL connect Lexer(scan tokens) → Parser(scan parse) → Evaluator(bus eval) → Serializer as a single reactive chain. Data SHALL flow: char → Token → AstNode → CssNode → String. The Reactor<S> type state machine ensures compile-time stage ordering.

#### Scenario: End-to-end compilation
- **WHEN** calling `from_string("$color: red; a { color: $color; }", options)`
- **THEN** the character stream flows through all stages and produces valid CSS output
- **AND** each stage's Observable consumes the previous output via ObservablePipe trait methods

#### Scenario: File-based compilation
- **WHEN** calling `Reactor::from_file(path)?.lex()?.parse()?.evaluate()?.serialize(style)?.finish()?`
- **THEN** reads source from file, compiles through all stages, returns CSS string

### Requirement: Reactor type state machine
The `Reactor<S>` struct SHALL use generic parameter `S` to encode the current pipeline stage. Each stage transition consumes self and returns `Result<Reactor<NextStage>>`. Invalid stage transitions are prevented at compile time.

#### Scenario: Stage ordering enforced
- **WHEN** attempting to call `.parse()` on a `Reactor<StateRaw>`
- **THEN** the code does not compile — `.parse()` is only available on `Reactor<StateLexed>`

### Requirement: CompilerBus for evaluation state
The evaluator stage SHALL use the existing CompilerBus (with SharedSubject channels) for evaluation-phase state sharing. Variables, mixins, functions, and scope events are communicated through the bus's pub-sub mechanism.

#### Scenario: Variable sharing between scopes
- **WHEN** a variable is declared in an outer scope and referenced in an inner scope
- **THEN** the evaluator publishes the binding through CompilerBus
- **AND** inner scope lookups resolve through the bus subscription

### Requirement: Pipeline error propagation
Errors from any stage SHALL propagate through the Observable error channel or the Result type. Lex errors, parse errors, and eval errors SHALL all be catchable at the pipeline level with descriptive messages including location information.

#### Scenario: Parse error propagation
- **WHEN** encountering a syntax error in the Parser stage
- **THEN** the error channel emits a Parse error variant, aborting the pipeline
- **AND** the error includes line:column position information

#### Scenario: Eval error propagation
- **WHEN** referencing an undefined variable
- **THEN** the pipeline returns an Eval error with the variable name and scope context

### Requirement: Reactive scheduling support
Each pipeline stage SHALL support `observe_on(ThreadPool)` for multi-threaded scheduling. The pipeline SHALL be composable — stages can be individually tested with mock input streams.

#### Scenario: Stage isolation
- **WHEN** testing the Parser stage with a hand-constructed token stream
- **THEN** the parser produces the expected AST without needing the Lexer to run

#### Scenario: Multi-threaded evaluation
- **WHEN** compiling a large SCSS file with many independent rules
- **THEN** the evaluate stage can use observe_on to parallelize rule evaluation

### Requirement: Load path resolution
The pipeline SHALL support `with_load_paths(paths)` for `@import` resolution. The resolver SHALL search: relative to current file, then each load path in order. SHALL support partials (`_filename.scss`) and multiple extensions.

#### Scenario: Import resolution
- **WHEN** compiling with include_path "scss/" and `@import "variables"`
- **THEN** resolves to "scss/_variables.scss" if it exists
