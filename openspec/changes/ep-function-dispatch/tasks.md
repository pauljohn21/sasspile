## 1. Diagnostic Instrumentation

- [ ] 1.1 Add `#[instrument]` to `eval_call` in value/mod.rs recording function name, arg count, and result
- [ ] 1.2 Add `#[instrument]` to `call_user_function` in mixin.rs recording params/args binding outcome
- [ ] 1.3 Add debug span in `call_function` namespace traversal logging which namespaces searched and match result
- [ ] 1.4 Add trace in `dispatch_function` UndefinedFunction fallback path recording the CSS passthrough conversion

## 2. Trace Collection

- [ ] 2.1 Create standalone test that compiles EP `button.scss` and emits full RUST_LOG=trace to /tmp/ep_button_trace.log
- [ ] 2.2 Analyze trace to identify at which step `joinVarName` / `getCssVar` / `bem` dispatch fails
- [ ] 2.3 Capture the exact state of `env.get_function()` and `env.get_namespaces()` at point of failure

## 3. Root Cause Analysis

- [ ] 3.1 Verify `eval_func_def` writes function to `env.local_functions` after parsing `@function`
- [ ] 3.2 Verify `@use` module loading populates `ModuleExports.all_functions()` correctly
- [ ] 3.3 Check FunctionDef `params.name` vs actual call arg binding for case-mismatch issues
- [ ] 3.4 Confirm whether EP uses `@import` (legacy) or `@use` (modern) and that the chosen path is exercised

## 4. Implementation Fix

- [ ] 4.1 Fix function dispatch to correctly find user-defined functions in current scope and namespaces
- [ ] 4.2 Fix List argument binding in function calls (when list is single arg vs spread)
- [ ] 4.3 Ensure function return value correctly propagates through `@return` node evaluation
- [ ] 4.4 Handle edge case: function defined after call in source order (Sass hoists @function definitions)

## 5. Regression Testing

- [ ] 5.1 Run `cargo test --test ep_dist_test` and record new pass count (baseline: 0/121 identical)
- [ ] 5.2 Run `cargo test --tests` all core tests, confirm 202/202 still pass
- [ ] 5.3 Run sass-spec full suite, confirm no regressions vs 8003/12133 baseline
- [ ] 5.4 Verify EP's most complex files (button.scss, table.scss, tabs.scss) produce matching output

## 6. Cleanup

- [ ] 6.1 Downgrade diagnostic spans from info to trace level
- [ ] 6.2 Remove temporary test artifacts (/tmp/ep_*_trace.log)
- [ ] 6.3 Finalize ep_dist_test.rs with proper assertions (currently runs without assert)
