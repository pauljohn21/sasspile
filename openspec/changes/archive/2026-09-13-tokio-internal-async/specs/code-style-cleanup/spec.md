## ADDED Requirements: Functional Rust Style Enforcement

### Scenario: for + push patterns replaced with iterator chains

- **GIVEN** any function uses `for x in items { result.push(f(x)) }` or similar patterns
- **WHEN** the cleanup pass is applied
- **THEN** the code is refactored to `items.into_iter().map(f).collect()` or `try_fold`
- **AND** no `let mut` accumulator appears before any for loop in collection-building contexts

### Scenario: if-else chains replaced with match

- **GIVEN** any function uses 3+ branch `if-else if-else` chain on enum or string matching
- **WHEN** the cleanup pass is applied
- **THEN** the code is refactored to `match` expression
- **AND** early returns for bool conditions (`match x { true => ... false => ... }`) are not affected

### Scenario: match Err return Err replaced with ?

- **GIVEN** any code pattern `match f() { Ok(v) => v, Err(e) => return Err(e.into()) }`
- **WHEN** the cleanup pass is applied
- **THEN** the code uses `f()?` instead of explicit match

### Scenario: Clippy zero warnings

- **GIVEN** `cargo clippy --all-targets` is run
- **WHEN** the cleanup is applied
- **THEN** all clippy warnings from the modified files are resolved (existing `#[allow(...)]` annotations preserved for pedantic lints)
