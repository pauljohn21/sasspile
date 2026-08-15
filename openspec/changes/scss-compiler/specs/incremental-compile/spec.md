## ADDED Requirements

### Requirement: Incremental Compiler SHALL debounce file system events
The compiler SHALL accumulate file change events and dispatch recompilation only after a configurable quiet period (default 200ms) to avoid thrashing during editor saves or batch operations.

#### Scenario: Single save triggers one compile
- **WHEN** user saves `app.scss` once (fsnotify fires 1-3 events within 50ms)
- **THEN** debouncer collapses events and triggers exactly one recompilation

#### Scenario: Multiple files edited in quick succession
- **WHEN** user saves 3 files within 300ms (e.g. IDE multi-file format)
- **THEN** debouncer waits 200ms after last change, then compiles all 3 files in one batch

#### Scenario: Continuous editing keeps resetting timer
- **WHEN** user types continuously (fsnotify fires every 100ms)
- **THEN** debouncer extends window; compile only triggers after 200ms of true quiescence

#### Scenario: Configurable debounce duration
- **WHEN** configured with `debounce_ms: 500`
- **THEN** compiler waits 500ms before dispatching recompile

#### Scenario: Force flush for CI
- **WHEN** `CompileMode::Ci` is set (or `--no-debounce` flag)
- **THEN** debounce is disabled; every event triggers immediate recompile

### Requirement: Incremental Compiler SHALL deduplicate identical events
When the same file path appears multiple times in the event buffer within a debounce window, it SHALL be deduplicated.

#### Scenario: Duplicate path collapse
- **WHEN** buffer contains `[("a.scss", 10ms), ("a.scss", 50ms), ("a.scss", 80ms)]`
- **THEN** deduplicated batch contains single `("a.scss", 80ms)` entry

### Requirement: Incremental Compiler SHALL use watch channels for variable propagation
When a variable changes, only downstream dependents SHALL be re-evaluated via `tokio::sync::watch`.

#### Scenario: Variable change triggers propagation
- **WHEN** `$primary: blue"` updates via channel
- **THEN** only rules referencing `$primary` are re-emitted; unrelated rules are untouched

#### Scenario: Watch channel absence
- **WHEN** variable is unused (no subscribers)
- **THEN** no re-evaluation happens (zero-cost idle)

### Requirement: Incremental Compiler SHALL cache AST nodes by span
Identical source spans with identical environment SHALL reuse previously computed values via `Arc`.

#### Scenario: Identical values reused
- **WHEN** same `@mixin` invoked twice with identical args
- **THEN** second call returns cached result, second evaluation skipped

#### Scenario: Source change invalidates cache
- **WHEN** an edit changes the source span of a cached node
- **THEN** new span invalidates the cache entry; re-evaluation occurs

### Requirement: Incremental Compiler SHALL track fine-grained dependencies
Each compiled node SHALL record which variables, mixins, and functions it depends on.

#### Scenario: Dependency registration
- **WHEN** rule `a { color: $c; }` is compiled
- **THEN** compiler registers dependency: `rule_a -> $c`

#### Scenario: Dependency removal on recompile
- **WHEN** rule is recompiled with no $c reference (edit)
- **THEN** old dependency `rule_a -> $c` is dropped from tracker

### Requirement: Incremental Compiler SHALL support partial pipeline restart
When a source file changes, the pipeline SHALL restart from the changed file forward, not from scratch.

#### Scenario: Single-file edit
- **WHEN** file `app.scss` changes
- **THEN** only `app.scss` and downstream importers are recompiled

#### Scenario: Library file changes
- **WHEN** library `_tokens.scss` changes and 3 files use it
- **THEN** tokens + 3 importers are recompiled, unrelated files untouched

### Requirement: Incremental Compiler SHALL expose reactive stream
Consumers SHALL subscribe to compilation results via `tokio::sync::watch::Receiver<CompileOutput>`.

#### Scenario: Subscribe before compile
- **WHEN** subscriber attaches to watch before compilation starts
- **THEN** subscriber sees initial `None` then subsequent `Some(CompileOutput)` values

#### Scenario: Late subscriber gets latest
- **WHEN** subscriber attaches after compilation completes and cached value exists
- **THEN** subscriber immediately receives the most recent `Some(CompileOutput)`
