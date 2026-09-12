## ADDED Requirements

### Requirement: No for-loop with push mutation
Collection transformations SHALL use iterator chains (`map`/`filter`/`flat_map`/`fold`/`collect`) rather than explicit `for` loops with mutable `Vec::push`.

#### Scenario: Accumulating parsed calc arguments
- **WHEN** splitting a comma-separated string into a `Vec<Value>`
- **THEN** the implementation uses `split_top_level(s).into_iter().map(parse_calc_arg_value).collect()` or equivalent iterator chain
- **AND** no local `let mut result = Vec::new()` with `result.push(...)` inside a `for` loop

#### Scenario: Splitting simple selectors
- **WHEN** splitting a compound selector string (e.g., `.foo.bar`) into simple selectors
- **THEN** the implementation uses an iterator-based approach (e.g., `peekable` + `fold`)
- **AND** no `let mut result = Vec::new()` with `for c in s.chars()` and `result.push(...)`

### Requirement: No match-with-return mixing
Functions SHALL NOT use `match` expression blocks containing `return` statements in some arms.

#### Scenario: Early-return parsing in call_hwb
- **WHEN** extracting通道 values from HWB constructor arguments
- **THEN** errors propagate via `?` operator and `Ok(Some(...))` wraps the success path
- **AND** no `match expr { Ok(x) => x, Err(e) => return Err(e), Ok(None) => return Ok(Some(...)) }` patterns

### Requirement: No repeated let-bindings with identical pattern
When 3 or more variables are assigned the same structural operation on different inputs, SHALL use `zip`+`map` or array iteration.

#### Scenario: RGB channel normalization in adjust_modern_rgb_space
- **WHEN** applying the same adjustment function to channels [0], [1], [2]
- **THEN** the implementation iterates via `keys.iter().zip(channels.iter()).map(...)` or array `[c0, c1, c2].iter()...`
- **AND** no `let r = f(ch[0]); let g = f(ch[1]); let b = f(ch[2]);` duplication

### Requirement: No duplicated structural match across functions
Functions with identical parameter validation logic SHALL extract a shared validation helper.

#### Scenario: call_extend and call_replace argument validation
- **WHEN** validating exactly N arguments for selector-extend and selector-replace
- **THEN** both functions delegate to `validate_exact_args(args, N, name)` or equivalent
- **AND** the `< N` and `> N` match arms appear only once

### Requirement: Semantic preservation
All refactored code SHALL preserve exact input/output semantics.

#### Scenario: Compilation tests pass unchanged
- **WHEN** running `cargo test --test compile_test --test stage_test --test ast_test --test common_test --test bs_spec --test interp_test`
- **THEN** all 200 tests pass with identical output

#### Scenario: sass-spec statistics do not regress
- **WHEN** running `SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture`
- **THEN** the pass count does not decrease compared to the pre-refactoring baseline
