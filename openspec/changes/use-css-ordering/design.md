## 核心问题

`@use` 和 `@import` 生成的 CSS at-rule（如 `@import "url"`）全部被移到文件顶部，而非保持在源码中的原始位置。

### 根因

1. `eval_hoist` 将所有 `@use`/`@forward`/`@import` 提前求值，生成的 CSS at-rule 被直接 push 到 `result` 前部
2. 流式 serializer 按顺序输出时，at-rule 与后续 rule 的相对位置被打乱

## 设计方案

### 1. CSS 节点保持原始位置

放弃 at-rule 集中前置策略，改为：

```
parse 阶段：@use/@import 生成占位 CssNode::ImportPlaceholder
eval 阶段：占位符替换为实际的 CssNode::AtRule
serialize 阶段：按 AST 原序输出
```

### 2. @import 嵌套展开

当 `@import` 嵌套在 CSS rule 内时：

```scss
a { @import "other"; }
→ a { /* other.scss 的内容展开到这里 */ }
```

- `eval_import` 返回 `Vec<CssNode>` 而非单个节点
- 调用处将返回节点注入到当前 rule 的 children 中

### 3. 占位符模式

```rust
enum CssNode {
    // ... 现有变体 ...
    /// @use/@import 在 eval 前为占位符，eval 后替换为实际内容
    PendingImport { url: String, namespace: Option<String> },
}
```

## 源文件修改

| 文件 | 修改 |
|------|------|
| `src/eval/hoist.rs` | 将 @use/@import 保留在原位而非提前 |
| `src/eval/import.rs` | eval_import 返回 Vec<CssNode> 而非单一节点 |
| `src/eval/rule.rs` | 处理嵌套 import 展开逻辑 |
| `src/css/serialize.rs` | 维持节点原序输出 |

## 验证

- `directives/use/css/order/*`（15 cases）
- `directives/import/css*`（12 cases）
- 核心测试 202/202 无回归
