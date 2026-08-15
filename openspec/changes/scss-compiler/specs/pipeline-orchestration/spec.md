## ADDED Requirements

### Requirement: Pipeline SHALL coordinate 7 Tokio stages via mpsc channels
The compiler SHALL spawn independent Tokio tasks for each stage: `Lexer → Parser → SemanticAnalysis → ExpressionEval → CssGen → IncrementalCache → Output`.

#### Scenario: Stage independence
- **WHEN** Lexer completes a token stream
- **THEN** Tokio sends to Parser via `mpsc::Sender`; Lexer can accept next input immediately

#### Scenario: Backpressure propagation
- **WHEN** Parser is slow (e.g., complex @mixin expansion)
- **THEN** Lexer's `mpsc::Sender::send()` awaits (bounded channel creates backpressure)

### Requirement: Pipeline SHALL be bounded for memory control
All inter-stage channels SHALL be bounded (configurable capacity) to cap memory per compilation.

#### Scenario: Default capacity
- **WHEN** pipeline uses default capacity of 64
- **THEN** each mpsc channel holds at most 64 items before blocking sender

#### Scenario: Small footprint config
- **WHEN** configured with `capacity: 8`
- **THEN** memory use is bounded even on very large SCSS files

### Requirement: Pipeline SHALL report progress via tracing
Each stage SHALL emit `tracing::info!` / `tracing::debug!` events for observability.

#### Scenario: Debug-level tracing
- **WHEN** RUST_LOG=`scss_compiler=debug`
- **THEN** tracing output shows per-token / per-ast-node activity

#### Scenario: Info-level tracing
- **WHEN** RUST_LOG=`scss_compiler=info`
- **THEN** tracing output shows only stage transitions and summary stats

### Requirement: Pipeline SHALL handle errors gracefully with stage recovery
A failure in one compilation SHALL not crash the entire pipeline; error SHALL be routed to diagnostics collector.

#### Scenario: Lexer error continues pipeline
- **WHEN** lexer hits invalid token
- **THEN** error is sent to diagnostics channel; pipeline completes with partial output + errors

#### Scenario: Panic isolation
- **WHEN** a Tokio task panics in Eval stage
- **THEN** `tokio::spawn` JoinHandle surfaces error; other stages continue or halt cleanly

### Requirement: Pipeline SHALL support concurrent file compilation
Multiple input files SHALL compile in parallel via separate pipeline instances or shared worker pool.

#### Scenario: Parallel compilation
- **WHEN** compiling `a.scss` and `b.scss` concurrently
- **THEN** each gets independent pipeline or dedicated Tokio tasks; results combined

#### Scenario: Shared module cache
- **WHEN** both files `@use` the same `_tokens.scss`
- **THEN** shared module store returns cached module; tokens are parsed and evaluated once

### Requirement: Pipeline SHALL be aborted cleanly
A `CancellationToken` SHALL allow aborting mid-compagation.

#### Scenario: Abort during long compile
- **WHEN** cancellation token is triggered
- **THEN** each stage checks token between messages; pipeline halts within one message of latency

#### Scenario: Pending work dropped
- **WHEN** abort triggered
- **THEN** in-flight messages are dropped; no partial-output is emitted to consumer
