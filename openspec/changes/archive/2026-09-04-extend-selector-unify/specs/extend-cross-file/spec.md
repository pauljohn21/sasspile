## Requirement: Module Scope Check Fallback

When `module_selectors` is empty or the extend's `module_path` is not found in the cache, the compiler MUST NOT skip the extend. Instead, it MUST apply the extend globally.

**Rationale**: Non-modular compilation (direct SCSS file compilation) produces `module = Some(base_path)` but `module_selectors` is empty. The current implementation incorrectly skips all extends in this scenario.

**Fix**: When `module_selectors.get(module_path)` returns `None`, fall through to applying the extend without scope restriction.

## Requirement: Extend Application via Selector AST

The `apply_extends` function MUST apply extends using the selector AST algebra (`extend_selector`), not string matching.

**Current bug**: The `fold` in `apply_extends` applies each extend sequentially, but module scope check causes early return for all extends in non-modular compilation.

## Requirement: Placeholder Selector Filtering After Extend

After applying extends, placeholder selectors (`%foo`) that were NOT extended MUST be removed from the output CSS. Placeholder selectors that WERE extended MUST also be removed (they serve as templates only).

**Current implementation**: Filters compounds where ALL simple selectors are `Placeholder(_)` — this is correct but only runs after extend application succeeds.
