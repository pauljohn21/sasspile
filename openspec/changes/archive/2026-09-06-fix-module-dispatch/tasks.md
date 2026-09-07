## 1. Fix `string_dispatch` name resolution

- [x] 1.1 In `string_dispatch` (line ~199-213), compute `global_name = string_builtin_name(name).unwrap_or(name)` before calling `call_string_builtin`

## 2. Fix `map_dispatch` name resolution

- [x] 2.1 In `map_dispatch` (line ~225-242), compute `global_name = map_builtin_name(name).unwrap_or(name)` before calling `call_map_builtin`

## 3. Fix `list_dispatch` name resolution

- [x] 3.1 In `list_dispatch` (line ~254-268), compute `global_name = list_builtin_name(name).unwrap_or(name)` before calling `list::call`

## 4. Fix `math_dispatch` name resolution

- [x] 4.1 In `math_dispatch` (line ~170-184), compute `global_name = math_builtin_name(name).unwrap_or(name)` before calling `math::call`

## 5. Fix `selector_dispatch` name resolution

- [x] 5.1 In `selector_dispatch` (line ~320-334), compute `global_name = selector_builtin_name(name).unwrap_or(name)` before calling `selector::call`

## 6. Verify and test

- [x] 6.1 Run core tests: `cargo test --test compile_test && cargo test --test stage_test && cargo test --test ast_test && cargo test --test common_test && cargo test --test bs_spec -- --nocapture && cargo test --test ep_full -- --nocapture` — all 202 MUST pass
- [x] 6.2 Run `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` — record pass rate, compare with baseline 6427/11824
- [x] 6.3 Verify no new errors from previously passing tests

## 7. Commit and sync

- [x] 7.1 `git add -A && git commit -m "fix: dispatch module-qualified names to canonical global names — 5 modules (string/map/list/math/selector)"`
- [x] 7.2 `codegraph sync`
- [x] 7.3 Wait for user confirmation before push
