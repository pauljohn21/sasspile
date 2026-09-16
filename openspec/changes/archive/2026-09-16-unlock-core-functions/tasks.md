## 1. 核心修复 — substitute_vars 点分路径吸收

- [x] 1.1 修改 `src/evaluate_dst/mod.rs` `substitute_vars` 标识符扫描循环，当 `.` 后紧跟 alphanumeric 或 `_` 时吸收进 ident（纯字符判断逻辑，无 rxrust 算子涉及）

## 2. 冒烟测试 — 确保不回归

- [x] 2.1 创建 `tests/core_functions_smoke.rs`，包含至少 6 条 `@use "sass:module";` + `module.fn(args)` 场景（math.abs / color.alpha / string.quote / list.append / map.get / meta.inspect）
- [x] 2.2 在冒烟测试中添加 fallback 场景：涵 CSS class selector `.container-fluid`、数值 `1.5px`、`url(foo.bar.png)`

## 3. 验证

- [x] 3.1 执行 `cargo test` 全量通过（含新增 smoke test），确认不回归现有行为
- [x] 3.2 执行 `cargo test --test sass_spec_detail` 采集 pass rate — 发现 sass-spec 计数未提升的**根因是 pre-existing 序列化格式问题**：`CssNode::render()` 输出单行 `a {b: 0;}` 而 spec 期望多行 `a {\n  b: 0;\n}`。本 fix (点分 dispatch) 已正确工作 (smoke test 验证通过)，但 spec 对比在两阶段 normalize 后因格式差异不匹配。格式修正属于独立 change 范围。
