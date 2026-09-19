# Tasks: arglist-each-iteration

## 1. 已修复 @each ArgList 迭代

- [x] **1.1** `src/eval/control_flow.rs` — `eval_each` 增加 ArgList 迭代支持
- [x] **1.2** `src/eval/value/ops.rs` — `add()` fallthrough 增强错误信息（用于诊断）

## 2. 二级修复: @extend 复杂选择器误判

### 问题
`eval_extend_node` 在插值前收到未求值的原始选择器字符串（如 `#{'%shared-' + $name}`），导致 `+` 运算符周围的空格误判为复杂选择器。

### 修复
- [x] **2.1** `src/eval/mod.rs` — `eval_extend_node` 在选择器检查前过滤 `#{...}` 片段

```rust
// 复杂选择器校验：跳过插值 #{...} 内部的空格（未插值占位符）
let scrubbed: String = target
    .split_inclusive('}')
    .filter(|chunk| !chunk.starts_with("#{"))
    .collect();
if scrubbed.chars().any(|c| c.is_whitespace()) {
    return Err(...);
}
```

## 3. 验证

- [x] **3.1** `cargo test --test ep_full -- --nocapture` 期望 121/121
- [ ] **3.2** 核心测试套件全量回归
- [ ] **3.3** `cargo clippy --all-targets` 零错误

## 实现约束（AI 必读）

- @each 改动：合并 match arm，零新逻辑
- @extend 改动：前置过滤字符串，不引入新依赖
- 所有测试必须 121/121 通过
