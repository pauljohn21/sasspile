## ADDED Requirements: Internal Async Module Loading

### Scenario: Module loading uses tokio async file IO

- **WHEN** the evaluator encounters `@use`, `@import`, or `@forward` with a file path
- **THEN** the file is read using `tokio::fs::read_to_string` (async IO)
- **AND** lex + parse + eval stages remain synchronous within the loaded module

### Scenario: evaluate_with_env becomes async

- **WHEN** `Evaluator::evaluate_with_env` is called
- **THEN** it is an `async fn` that may await module loading
- **AND** the function signature changes from `fn(...) -> Result<Vec<CssNode>>` to `async fn(...) -> Result<Vec<CssNode>>`

### Scenario: Reactor::evaluate becomes async

- **WHEN** `Reactor::evaluate()` is called
- **THEN** it returns `impl Future<Output = Result<Reactor<StateEvaluated>>>`
- **AND** callers must `.await` the result before proceeding

### Scenario: Public API stays synchronous

- **WHEN** a user calls `sasspile::compile(input, style)`
- **THEN** the function signature remains `fn(&str, OutputStyle) -> Result<String>` (synchronous)
- **AND** inside the function, a global tokio runtime's `block_on` drives the async pipeline
- **AND** no .await is required from the consumer

### Scenario: Compatible with non-tokio contexts

- **WHEN** sasspile is used inside an existing tokio runtime (e.g., user's async codebase)
- **THEN** the global runtime is still initialized independently (sasspile manages its own)
- **AND** the `block_on` call works correctly without nested runtime errors

---

## ADDED Requirements: Reactor Dead Code Removal

### Scenario: ModuleCacheEntry struct is removed

- **GIVEN** `src/eval/reactor_types.rs` previously defined `ModuleCacheEntry`
- **WHEN** the cleanup is applied
- **THEN** the struct definition is removed
- **AND** all references to `ModuleCacheEntry` are removed (only `Reactor.modules` field)

### Scenario: Reactor dead fields are removed

- **GIVEN** `Reactor` struct had unused fields
- **WHEN** cleanup is applied
- **THEN** the following fields are removed: `env`, `modules`, `io_log`, `warnings`, `io`
- **AND** all stage transitions (lex/parse/evaluate/serialize) no longer carry these fields
- **AND** `new()` and `from_file()` constructors no longer initialize these fields

### Scenario: ReactorIO trait and impls are removed

- **GIVEN** `ReactorIO`, `DefaultReactorIO`, `MockReactorIO` were defined in `reactor_types.rs`
- **WHEN** cleanup is applied
- **THEN** the trait and both structs are removed
- **AND** the `pub use` exports in `lib.rs` are removed
- **AND** tests using `MockReactorIO` are removed

### Scenario: Warning and IoRecord types are removed

- **GIVEN** `Warning` and `IoRecord` structs in `reactor_types.rs`
- **WHEN** cleanup is applied
- **THEN** both structs are removed

### Scenario: ReactorTrace preserves pipeline entry time

- **GIVEN** `ReactorTrace::advance()` previously created a new `Instant::now()`
- **WHEN** the fix is applied
- **THEN** `advance()` preserves `self.entered_at` from the trace where the pipeline entered
- **AND** the `finish()` debug log accurately reports total_elapsed_us from pipeline start

---

## MODIFIED Requirements: Parser Stream Simplification

### Scenario: ParseStream implements Iterator instead of tokio_stream::Stream

- **GIVEN** `ParseStream` previously implemented `tokio_stream::Stream` with `Pin`/`Context`/`Poll`
- **WHEN** the refactor is applied
- **THEN** `ParseStream` implements `std::iter::Iterator<Item = Result<Node>>`
- **AND** `poll_next`, `Context`, `Pin` imports are removed
- **AND** `FusedStream` impl is removed
- **AND** `Parser::parse` uses standard `.collect()` instead of `futures::executor::block_on_stream`

### Scenario: Cargo dependencies updated

- **GIVEN** `Cargo.toml` had `tokio-stream = "0.1"` and `futures = "0.3"`
- **WHEN** the refactor is applied
- **THEN** these are replaced with `tokio = { version = "1", features = ["full"] }`
- **AND** the project compiles without `tokio-stream` or `futures` crates
