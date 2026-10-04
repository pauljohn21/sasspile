# Proposal: Grass Reactive Compiler

## Why

The current grass Sass compiler uses a traditional pull-based imperative architecture where all intermediate products (tokens, AST, CSS statements) are fully materialized in memory through shared mutable state (`Rc<RefCell<...>>`). This prevents streaming output, makes parallel consumption impossible, and tightly couples all `@`-directive handlers through a single 2500-line `Visitor` match. By rebuilding on rxrust's Observable primitives, the compiler gains streaming output (first CSS bytes emitted before full compilation completes), multicast fan-out (multiple independent consumers of tokens/AST/CSS), and a Rust-ownership-native architecture where each `@`-directive is an independent operator with its own lifecycle.

## What Changes

- Replace the `Lexer` pull-based token buffer with an `Observable<char> → Observable<Token>` stream
- Replace the recursive-descent `Parser` with a state-machine wrapped in `scan`/`flat_map` producing `Observable<AstNode>`
- Replace the monolithic `Visitor` with trait-based `SassOp` operators: each `@`-directive (and each statement type) implements `SassOp`, returning an independent `Observable` transformer
- Introduce `CompilerBus` — a set of multicast `Subject` streams (`ast_nodes`, `var_events`, `css_stream`, `module_events`, `diag_events`) that serve as the wiring layer between operators
- Replace `Rc<RefCell<Environment>>` with `Rc<EvalContext>` (immutable-shared) where state changes emit `VarEvent` onto a Subject rather than mutating shared memory
- Add streaming API `from_string_stream() → Observable<char>` alongside the existing `from_string() → Result<String>` (backward compatible)
- Replace `CssTree` tombstone-based tree construction with `scan`/`buffer` operators on the multicast `css_stream`

## Capabilities

### New Capabilities

- `reactive-pipeline`: Core Observable-based compilation pipeline that transforms input characters through Lexer → Parser → Evaluator → Serializer stages connected by multicast streams, supporting streaming output and parallel consumption
- `directive-ops`: Trait-based operator system where each Sass `@`-directive and statement type is an independent `SassOp` implementation that transforms Observables, enabling composable and extensible directive semantics
- `multicast-bus`: Shared event bus (`CompilerBus`) of hot Observables/Subjects that decouples producers from consumers, enabling one compilation to feed multiple independent output streams simultaneously

### Modified Capabilities

None. This is a greenfield architecture for the Lightforger workspace — no existing specs to modify.

## Impact

- **Code**: New crate `grass-reactive` in the workspace (alongside existing crates); existing grass code in `/Users/panglijun/rust/grass` serves as reference implementation
- **Dependencies**: Adds `rxrust = "1.0.0-rc.5"` as core dependency
- **API**: New `from_string_stream()` function; existing `from_string()`/`from_path()` preserved for backward compatibility
- **Architecture**: Fundamental shift from pull-based visitor to push-based reactive streams; directly affects how all Sass language features are implemented
- **Performance**: Streaming output reduces time-to-first-byte; multicast avoids duplicate work for multi-consumer scenarios
