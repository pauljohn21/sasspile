//! Extension combinators for observable streams in the Sass compilation
//! pipeline (Design §Decision 3).
//!
//! Placeholder module — the concrete implementations will be added once
//! the `CompilerBus` and `AstStream`/`CssStream` types are exercised in
//! integration tests (task 3.1 + 3.2 in the task list).

// Reserved for future implementation of:
//   - `flat_map_extract_items` — flatten Observable<Vec<T>> → Observable<T>
//   - `switch_map_extract_items` — same with cancellation semantics
