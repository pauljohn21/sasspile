---
description: src/css CSS 构建模块专用规则
---

# src/css/ CSS 构建模块规则

## 借用优先

```rust
// ✅ 借用
impl CssNode {
    fn render(&self, builder: &mut CssBuilder) { ... }
    fn serialize(&self) -> String { ... }
}
```

## 禁止 Rc<RefCell> 共享可变

```rust
// ✅ CssBuilder 由 scan_map 持有
source.scan_map(CssBuilder::default(), |builder: &mut _, line: &str| {
    builder.parse_line(line);
    builder.output()
})
.flat_map(|v: Vec<&str>| Local::from_iter(v))
```

## tracing span

```rust
fn render_css(node: &CssNode) -> String {
    let _span = info_span!("render_css",
        variant = %std::any::type_name_of_val(node)
    ).entered();
    // ...
}
```

## 单文件 ≤ 500 行

src/css/ 下的 builder.rs / node.rs 各自不超过 500 行。
