## 1. 顶层裸声明检测

- [ ] 1.1 在 `src/eval/mod.rs` 的 `eval_node` 的 `Node::Decl` 分支中，在 plain CSS 检查之后、值求值之前，增加 `env.current_selector.is_none() && !env.plain_css` 检查，报错 `expected "{".`
- [ ] 1.2 运行 `RUST_LOG=trace cargo test --test sass_spec_full` 验证 Group 2（root/input.scss → `a: b;`）报错

## 2. 顶层 @include 声明检测

- [ ] 2.1 在 `src/eval/mixin.rs` 的 `eval_include` 中，在 `exec_mixin` 返回后检查 CSS 输出是否包含 `CssNode::Declaration`，且 `env.current_selector.is_none()`，报错 `Declarations may only be used within style rules.`
- [ ] 2.2 运行 `RUST_LOG=trace cargo test --test sass_spec_full` 验证 Group 1（include/input.scss → `@include a;`）报错

## 3. 回归测试

- [ ] 3.1 运行 `cargo test --test compile_test`（43 个核心测试不回归）
- [ ] 3.2 运行 `cargo test --test ep_full`（121 个不回归）
- [ ] 3.3 运行 `cargo test --test sass_spec_full`（@directives import 通过率提升，无新回归）
- [ ] 3.4 验证 `top_level_parent.hrx`（`& {a: b}` 顶层——合法，不应报错）
