## Context

EP consistency at 83/121 (68.8%). Previous phases 2026-09-22 → 2026-10-02 achieved:
- BEM mixin context fix (save/restore current_selector)
- @at-root / @content ordering
- @extend %placeholder selector grouping
- `:not()` + `:is()` bracket-aware split (NEW — Phase 2)
- parse_literal_arg 1-char safety (NEW — Phase 2)

Remaining 38 DIFF files fall into two classes:
1. **Sasspile bugs** (~5 files): Compile-time logic errors, fixable in sasspile source
2. **Pipeline features** (~33 files): Autoprefixer, lightningcss minification, lightningcss color-scheme — not reproducible in sasspile's raw output

### Sasspile Bugs (actionable)
| File | Root Cause |
|------|-----------|
| descriptions.scss | `e(title)` inside `m($size)` loses @at-root enclosure → `.el-descriptions--large .el-descriptions__title` instead of `.el-descriptions--large .el-descriptions__header .el-descriptions__title` |
| input-series | `rgba()` call args with var() not expanded → combined selector split |

### Pipeline Features (non-actionable in sasspile)
All remaining 33 files: `-webkit-user-select`, `translate(0)` simplification, `calc(1px * 2)` simplification, `--lightningcss-light:initial`.

## Goals / Non-Goals

**Goals:**
- Fix all actionable sasspile bugs (descriptions header nesting, input-number var() expand)
- Maintain sass-spec at ≥7927, core tests 202/202
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

**Problem**: EP output is `.el-descriptions--large .el-descriptions__header .el-descriptions__title`, sasspile emits `.el-descriptions--large .el-descriptions__title` (missing `__header` layer). Root cause: `e(title)`'s `@at-root` inside m($size) body fails to inherit the enclosing `__header` selector.

**Decision**: In `eval_rule`, when `e()` mixin emits `@at-root { .#{$currentSelector} { ... } }`, the `$currentSelector` should be prefixed by the current `env.current_selector` chain if the parent selector contains a modifier prefix that isn't `&`.

### Decision 3: Pipeline-feature DIFF marking

**Problem**: 33 remaining files have DIFF caused by dart-sass+lightningcss features not in sasspile (autoprefixer, minification, color-scheme).

**Decision**: Mark these as `KNOWN_DIFF` category in `ep_normalized_test.rs` with comment markers. These are not sasspile bugs. Future EP-phase work may normalize them via post-processing but this is out-of-phase.

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| descriptions fix affects other BEM `e(m())` combos | SPEC_STORE_CMD=run 全量验证 |
| at-root selector inheritance logic over-broad | Guard flag: only when parent chain has non-trailing `&` entry |
| Pipeline-feature normalization changes sass-spec | Only apply when EP-mode compile flag set |
