---
description: tests 目录测试专用规则
---

# tests/ 目录规则

## 必须遵守

### 1. 测试中允许 expect() 但要有信息

```rust
let result = compile(input).expect("compile should succeed for basic SCSS");
```

### 2. 用 #[test] 不用内联测试

```rust
#[test]
fn test_basic_variable() {
    let result = compile("$color: red; .x { color: $color }");
    assert_eq!(result, ".x {\n  color: red;\n}\n");
}
```

### 3. 反应式管线测试模式

```rust
#[test]
fn test_reactive_pipeline() {
    let mut collected = vec![];

    Local::from_iter(input.lines())
        .map(|line: &str| line.trim())
        .filter(|line: &&str| !line.is_empty())
        .scan_map(0_usize, |count: &mut _, line: &str| {
            *count += 1;
            format!("/* line {} */ {}", count, line)
        })
        .collect::<Vec<String>>()
        .last()
        .subscribe(|v: Vec<String>| collected = v);

    assert_eq!(collected, vec!["/* line 1 */ body { color: red; }"]);
}
```

## ⛔ 禁止

- 在 tests/ 中 import src/ 私有项（集成测试只能访问 pub API）
- 用 `dbg!()` 宏（用 `tracing::debug!` 替代）
- 跨文件共享可变状态
- `unwrap()` 在 src/ 中（只在 tests/ 允许 `expect("原因")`）
