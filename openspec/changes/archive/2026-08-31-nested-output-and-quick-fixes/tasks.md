# Tasks: Nested Output + Quick Fixes

## Phase 1: Quick Fixes

- [ ] 1. Fix string-to-number coercion in math functions (`"0"` → `0.0`)
- [ ] 2. Fix `Unsupported + operation` for Calc/string concatenation
- [ ] 3. Fix `Unsupported - operation` for Calc expressions
- [ ] 4. Fix `if` function to accept exactly 3 arguments
- [ ] 5. Fix `set-nth` argument validation
- [ ] 6. Fix `rgba` to accept 3-4 number arguments properly
- [ ] 7. Run tests and verify Phase 1 pass rate improvement

## Phase 2: Nested Output Format

- [ ] 8. Refactor `RuleBuilder::push` to preserve nested children instead of flattening
- [ ] 9. Delay `&` selector replacement to serialization (preserve `&` in CssNode)
- [ ] 10. Update `Serializer::serialize` to skip `flatten_nodes` in expanded mode
- [ ] 11. Verify `@media` nested rules preserve structure
- [ ] 12. Verify `@at-root` content hoists to top level
- [ ] 13. Run compile_test + ep_full to verify no regression
- [ ] 14. Run sass_spec_full and measure pass rate improvement

## Verification

- [ ] 15. Final sass-spec full test run
- [ ] 16. codegraph sync + commit
