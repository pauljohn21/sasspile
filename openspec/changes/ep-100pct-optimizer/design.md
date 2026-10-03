## Context

EP consistency at 106/121 (87.6%). Previous phases 2026-09-22 → 2026-10-03 achieved:
- BEM mixin context fix (save/restore current_selector)
- @at-root / @content ordering
- @extend %placeholder selector grouping
- `:not()` + `:is()` bracket-aware split (NEW — Phase 2)
- parse_literal_arg 1-char safety (NEW — Phase 2)
- at_root_top flag + compose-in-descender for depth>0 (NEW — Phase 3)

Remaining 15 DIFF files fall into two classes:
1. **Sasspile bugs** (~4 files): Compile-time logic errors, fixable in sasspile source
2. **Pipeline features** (~11 files): Autoprefixer, lightningcss minification, lightningcss color-scheme — not reproducible in sasspile's raw output

### Sasspile Bugs (actionable)
| File | Root Cause |
|------|-----------|
| input-series | `rgba()` call args with var() not expanded → combined selector split |

### Pipeline Features (non-actionable in sasspile)
All remaining 11 files: `-webkit-user-select`, `translate(0)` simplification, `calc(1px * 2)` simplification, `--lightningcss-light:initial`.

## Goals / Non-Goals

**Goals:**
- Fix all actionable sasspile bugs (input-number var() expand)
- Maintain sass-spec at ≥7927, core tests 119/119
- Document pipeline-feature DIFF as non-actionable baseline

**Non-Goals:**
- Simulate autoprefixer vendor-prefix insertion in sasspile output
- Backport lightningcss translate/calc simplification
- Implement lightningcss custom-media color-scheme vars

## Decisions

### Decision 1: `:not()` bracket-aware comma split in `combine_selectors`

**Problem**: `combine_selectors(parent, "&:hover:not(.a, .b)")` split on commas, treating `:not(.a` and `.b)` as separate selectors.

**Decision**: Add `split_selectors_respecting_parens` that tracks `()` nesting depth. Commas inside nested parens are not treated as selector separators.

**Validation**: color-picker `:not()` produces semantically-correct output. sass-spec +6.

### Decision 2: descriptions `e(title)` at-root enclosure strategy

**Problem**: EP output is `.el-descriptions--large .el-descriptions__header .el-descriptions__title`, sasspile emitted `.el-descriptions--large .el-descriptions__title` (missing `__header` layer). Root cause: `e(title)`'s `@at-root` inside m($size) body fails to inherit the enclosing `__header` selector.

**Decision**: Three-part fix:
1. **`at_root_top` flag** (new field on `Env`): Set to `true` in `eval_at_root()` entry point. Indicates we're inside @at-root context.
2. **Compose-in-descender with depth guard**: In `eval_rule()` selector resolution, when `depth > 0` and NOT `at_root_top`, always run `combine_selectors(parent, selector)` — even for selectors without `&`. This ensures nested elements get the full prefix chain.
3. **`starts_with` prefix detection**: In `RuleBuilder::push()`, detect when a child Rule's selector already starts with the parent's selector. Skip composition to avoid double-prefixing (e.g., ____large--large__header).

**Validation**: descriptions.scss output matches EP dist exactly. Core tests 119/119, sass-spec +0 (no regression).

### Decision 3: Pipeline-feature DIFF marking

**Problem**: 11 remaining files have DIFF caused by dart-sass+lightningcss features not in sasspile (autoprefixer, minification, color-scheme).

**Decision**: Mark these as `KNOWN_DIFF` category in `ep_normalized_test.rs` with comment markers. These are not sasspile bugs. Future EP-phase work may normalize them via post-processing but this is out-of-phase.

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| descriptions fix affects other BEM `e(m())` combos | SPEC_STORE_CMD=run 全量验证 |
| at-root selector inheritance logic over-broad | Guard: only compose on depth > 0, skip on top-level (extend safety) |
| Pipeline-feature normalization changes sass-spec | Only apply when EP-mode compile flag set |
| starts_with check too broad | Only triggered when child EXACTLY starts with parent selector (trim whitespace) |
