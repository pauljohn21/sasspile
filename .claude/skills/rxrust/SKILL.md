---
name: rxrust
description: Use when working with rxrust reactive streams, Observable patterns, or designing data processing pipelines. Covers Observable/Observer/Subscription triad, Local vs Shared context, operators (map, filter, flat_map, expand, scan, merge), type erasure (box_it), Subscription lifecycle, and OpenTelemetry tracing integration via tracing-subscriber. Use when the user mentions rxrust, reactive streams, Observable, SharedSubject, or when designing stream-based data pipelines in Rust.
---

# RxRust Skill

## Core Philosophy

RxRust is **Push-based** composition. Unlike Iterator (pull-based), the source decides when to emit. Think of data as flowing through a pipeline — each stage consumes from upstream and emits downstream.

**Rust ownership matters**: Observables own their data. Calling an operator (e.g., `map`) *consumes* (moves) the original observable into a new one.

## The Triad

| Role | rxRust Type | Responsibility |
|------|-------------|----------------|
| Observable | `Observable` trait | The lazy data source. Nothing happens until subscribed. |
| Observer | closure in `.subscribe(\|v\| ...)` | Reacts to `next`, `error`, `complete`. |
| Subscription | return of `.subscribe()` | Controls stream lifecycle. |

## Context: Local vs Shared

| Context | Use When | Internals | Item Bounds |
|---------|----------|-----------|-------------|
| `Local` | Single-thread (WASM, UI main) | `Rc<RefCell>` — zero lock | None |
| `Shared` | Multi-thread server | `Arc<Mutex>` — work-stealing | `Send + Sync` |

**Rule of thumb**: rx-scss uses `Shared` (static lifetimes, multi-thread capable). Type aliases in `types.rs`:
```rust
pub type TokenStream = SharedBoxedObservable<'static, Token, Infallible>;
pub type AstStream = SharedBoxedObservable<'static, AstNode, Infallible>;
pub type CssStream = SharedBoxedObservable<'static, CssStmt, Infallible>;
```

## Type Erasure — `box_it()`

Operators create deeply nested types. Use `.box_it()` to erase to `SharedBoxedObservable`:
```rust
Shared::create(move |subscriber| { ... }).box_it()
```

**Always end a pipeline stage with `.box_it()`** when returning a trait object type.

## Creating Observables

```rust
// From closure
Shared::create(move |subscriber| {
    subscriber.next(value);
    subscriber.complete();
}).box_it()

// From iterator
Shared::from_iter(vec).box_it()

// Subject (hot observable — manual emit)
let subject: SharedSubject<'static, T, Infallible> = Shared::subject();
subject.next(value);      // emit
subject.complete();       // signal done
subject.box_it()          // convert to Observable for chaining
```

## Key Operators for Pipeline Design

### Transformation
| Operator | Signature | Use Case |
|----------|-----------|----------|
| `map` | `T -> U` | Transform each item |
| `flat_map` | `T -> Observable<U>` | 1-to-many, merge results |
| `expand` | `T -> Observable<T>` | Recursive unfold — emit children back into same stream |
| `scan` | `(State, T) -> (State, U)` | Accumulate state, emit intermediate results |
| `filter_map` | `T -> Option<U>` | Map + filter in one step |
| `reduce` | `(State, T) -> State` | Fold to single value |

### Filtering
| Operator | Use Case |
|----------|----------|
| `filter` | Predicate-based selection |
| `take(n)` | First n items |
| `skip(n)` | Skip first n items |
| `distinct_until_changed` | Deduplicate consecutive |

### Combination
| Operator | Use Case |
|----------|----------|
| `merge` | Interleave multiple streams |
| `zip` | Pairwise combine |
| `start_with` | Prefix values |

## Subscription Lifecycle

```rust
// Manual unsubscribe
let sub = stream.subscribe(|v| { ... });
sub.unsubscribe();

// RAII (auto-unsubscribe on drop)
let sub = stream.subscribe(|v| { ... });
let _guard = sub.unsubscribe_when_dropped();  // hold this guard!
// stream cancelled when _guard drops
```

## Pattern: Reactive Recursion (expand)

For tree-structured data (AST, nested rules), use `expand` instead of recursive functions:

```rust
// Concept: emit Enter/Leave events, fold into tree via scan
ast_stream
    .expand(|node| match node {
        Compound(children) => Shared::from_iter(children),  // re-inject children
        Terminal(_) => Shared::empty(),                      // leaf — no expansion
    })
    .scan(initial_state, |state, event| {
        // fold events into nested structure
    })
```

This replaces deep recursion with bounded stream processing — no stack overflow.

## OpenTelemetry Tracing Integration

rx-scss uses `tracing` + `tracing-subscriber` for structured telemetry. Initialize in your entry point or test:

```rust
// Production: stdout with line numbers
rx_scss::telemetry::init_tracing();

// Test: stderr output with --nocapture support
rx_scss::telemetry::init_test_tracing();
```

### Span Creation Patterns

```rust
// Preferred: #[instrument] on functions
#[tracing::instrument(skip(bus), fields(node_count = nodes.len()))]
fn eval_nodes(nodes: Vec<AstNode>, bus: &CompilerBus) -> Result<Vec<CssStmt>> {
    // ...
}

// Alternative: inline span with .entered()
let _span = tracing::info_span!("parse_at_if").entered();
// ... logic ...
// _span drops → exit logged

// Debug event inside span
tracing::debug!(?cond_val, is_truthy, "condition evaluated");
```

### Span Field Sigils

| Sigil | Format | Use For |
|-------|--------|---------|
| `?` | Debug | Complex types like `Value`, `AstNode` |
| `%` | Display | User-facing strings like selectors |
| (none) | Value trait | Primitive types (bool, u32, etc.) |

### Environment Configuration

```bash
# Filter spans via RUST_LOG
RUST_LOG=debug cargo test --test telemetry_test -- --nocapture
RUST_LOG=rx_scss=trace cargo run -- input.scss

# Available levels: error, warn, info, debug, trace
```

## Pattern: Subject as Channel

Use `SharedSubject` for cross-component communication (events, state changes):

```rust
let var_subject: SharedSubject<'static, VarEvent, Infallible> = Shared::subject();

// Producer
var_subject.next(VarEvent::Bind { ... });

// Consumer
var_subject.subscribe(|event| { ... });

// Convert to chained Observable
var_subject.filter_map(|e| { ... }).box_it()
```

## Anti-Patterns to Avoid

| ❌ Anti-Pattern | ✅ Correct Approach |
|---|---|
| Recursive function on tree nodes | `expand` + `scan` stream folding |
| `Arc<Mutex<T>>` for single-thread | `Local` context (zero-cost) |
| Manual callback hell | Declarative operator chain |
| Holding Observable without subscribing | Subscribe to activate (lazy) |
| Collecting entire stream into Vec | Stream processing (filter, map, fold) |
| `println!` inside stream ops | `tap` operator for side effects + tracing `debug!` |
| `std::fs::write` for debug logs | `tracing::debug!` with `RUST_LOG=debug` |

## Quick Reference: rx-scss Pipeline

```
Source → scan() → TokenStream → parse_stream() → AstStream → eval_stream() → CssStream → serialize() → CSS
```

Each stage consumes `SharedBoxedObservable<'static, T, Infallible>` from previous stage.
