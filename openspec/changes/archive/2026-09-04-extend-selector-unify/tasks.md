# Tasks: extend-selector-unify

## Phase 1: Module Scope 修复（P0 — 阻断所有 extend）

- [x] T1: 修复 `src/eval/extend.rs` 中 `apply_extends` 的 module scope 检查：当 `module_selectors` 为空或 `module_path` 不在缓存中时，不跳过 extend（降级为全局匹配）
- [x] T2: 验证 `a {b: c} d {@extend a}` 输出 `a, d { b: c; }`
- [x] T3: 运行 `cargo test --test compile_test` 确认无回归

## Phase 2: 逗号分隔多目标拆分（P1）

- [x] T4: 在 `src/eval/mod.rs` 的 `eval_extend_node` 中按 `,` 拆分 target，为每个目标生成独立 extend 条目
- [x] T5: 验证 `@extend .a, .b` 等价于 `@extend .a; @extend .b;`
- [x] T6: 运行 `cargo test --test compile_test` 确认无回归

## Phase 3: Extend 参数校验（P2）

- [x] T7: 在 `eval_extend_node` 中新增复杂选择器校验：target 包含多个 compound（空格分隔）→ 报错 "complex selectors may not be extended."
- [x] T8: 新增复合选择器校验：target 包含伪类/伪元素且非纯伪类 → 报错 "compound selectors may no longer be extended. Consider `@extend a, :hover` instead."
- [x] T9: 新增空选择器校验：target 为空 → 报错 "expected selector."
- [x] T10: 运行 sass-spec 确认 `directives/extend/error` 用例通过

## Phase 4: fold 去重重构（P3 — 函数式规范）

- [x] T11: 重构 `src/eval/extend.rs` 中 `apply_extends` 的 fold 去重为函数式写法（禁止可变 Vec + push）
- [x] T12: 重构 `src/css/selector_ops.rs` 中 `extend_selector` 和 `replace_selector` 的 fold 去重为函数式写法
- [x] T13: 运行 `cargo test --test compile_test` 和 `cargo test --test stage_test` 确认无回归

## Phase 5: 全量验证

- [x] T14: 运行 `cargo test --test compile_test` — 43/43
- [x] T15: 运行 `cargo test --test stage_test` — 10/10
- [x] T16: 运行 `cargo test --test ep_full -- --nocapture` — 121/121
- [x] T17: 运行 `RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture` 确认 `directives/extend` 通过率提升
- [x] T18: `codegraph sync` 更新索引
