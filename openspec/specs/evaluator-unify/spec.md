## ADDED Requirements

### Requirement: Single Evaluator type for all Sass compilation
The compiler SHALL expose exactly one `Evaluator` type that handles both SCSS and SASS-downgraded-to-SCSS sources. There SHALL NOT be separate `ScssEvaluator` / `CssEvaluator` types.

#### Scenario: No ScssEvaluator in public API
- **WHEN** external code imports from `sasspile::eval`
- **THEN** only `Evaluator` is available; `ScssEvaluator` does not exist

#### Scenario: Evaluator compiles SCSS source correctly
- **WHEN** `Evaluator::evaluate(&ast)` is called with a parsed SCSS AST
- **THEN** it produces the correct `Vec<CssNode>` output

### Requirement: Reactor uses Evaluator directly
The `Reactor<StateParsed>::evaluate` method SHALL call `Evaluator::evaluate_with_env` directly instead of going through `ScssEvaluator` intermediaries.

#### Scenario: Reactor evaluate calls Evaluator
- **WHEN** `Reactor::evaluate()` is invoked
- **THEN** it calls `Evaluator::evaluate_with_env(&ast, env)` without intermediate wrapper

### Requirement: Consolidation preserves behavior
Merging `ScssEvaluator` into `Evaluator` SHALL NOT change any external API signatures or compilation behavior. Existing tests MUST continue to pass.

#### Scenario: All core tests pass after consolidation
- **WHEN** `cargo test --test compile_test && cargo test --test stage_test && cargo test --test ep_full` run after consolidation
- **THEN** all tests pass (202/202 for compile+stage+ast+common+bs+ep baseline)
