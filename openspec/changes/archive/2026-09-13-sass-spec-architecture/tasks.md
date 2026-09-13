## 1. Fix CSS import pass-through in eval_import

- [x] 1.1 Add `path.extension().is_some_and(|e| e == "css")` check in `eval_import` after resolve, returning early with `@import url(...)` node instead of calling `load_import`
- [x] 1.2 Add test in `compile_test.rs` verifying `@import "existing.css";` outputs `@import url("existing.css");` and does NOT parse the CSS file
- [x] 1.3 Run `cargo test --test compile_test` and verify all existing tests pass

## 2. Add explicit errors for @use/@forward of CSS

- [x] 2.1 In `load_module`, add URL suffix check: if URL ends with `.css`, return `SassError::Eval("CSS files can't be @used")`
- [x] 2.2 In `eval_forward` (forward.rs), add equivalent `.css` check with error "CSS files can't be @forwarded"
- [x] 2.3 Add tests verifying `@use "foo.css"` and `@forward "foo.css"` produce clear error messages
- [x] 2.4 Run `cargo test --test compile_test` to confirm no regression

## 3. Merge ScssEvaluator into Evaluator

- [x] 3.1 Change `Reactor::evaluate` in `reactor.rs` to call `Evaluator::evaluate_with_env(&ast, env)` directly (remove `ScssEvaluator::` prefix)
- [x] 3.2 Remove `pub use scss_evaluator::ScssEvaluator;` from `eval/mod.rs`
- [x] 3.3 Remove `pub use scss_evaluator::ScssEvaluator;` re-export from `lib.rs`
- [x] 3.4 Delete `src/eval/scss_evaluator.rs` file
- [x] 3.5 Run `cargo test --test reactor_test` to verify Reactor still works
- [x] 3.6 Run `cargo test --test compile_test --test stage_test --test ast_test` for core test baseline

## 4. Test framework file role classification

- [x] 4.1 Modify `parse_hrx_to_cases` in `hrx_support.rs` to classify files by role (Input/Output/ScssAsset/CssAsset)
- [x] 4.2 Modify `run_case` to skip writing `output.css` content to temp directory (only write Input + ScssAsset + CssAsset)
- [x] 4.3 Update `all_files` generation: exclude `output.css` from the `files` vector written to VFS
- [x] 4.4 Run `SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture` to verify no regression
- [x] 4.5 Verify css-asset files (non-output .css) are still written to VFS for @import resolution

## 5. Verification and baseline check

- [x] 5.1 Run full core test suite: `cargo test --test compile_test && cargo test --test stage_test && cargo test --test ast_test && cargo test --test common_test && cargo test --test bs_spec && cargo test --test ep_full` (must pass 202/202)
- [x] 5.2 Run `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` to check no spec regressions
- [x] 5.3 Document any spec changes in openspec delta if needed（无 spec 行为变化，纯正确性修复）
