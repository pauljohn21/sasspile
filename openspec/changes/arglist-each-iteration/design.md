# 修复 @each 无法迭代 ArgList

## 问题

element-plus 全量测试（`ep_full`）当前 119/121，2 个文件失败。

### 直接症状

`index.scss` 和 `table.scss` 触发 `"complex selectors may not be extended."` 错误，
但根本原因不同——真正导致 element-plus 编译出错的是 **`@each` 无法迭代 `ArgList`**。

### 根因代码

`src/eval/control_flow.rs::eval_each` 第 133-154 行：

```rust
let items: Vec<Vec<Value>> = match &evaluated {
    Value::Map(pairs) if vars.len() >= 2 => ...,
    Value::Map(pairs) if vars.len() == 1 => ...,
    Value::List(es, _, _) => es.iter().map(|e| vec![e.clone()]).collect(),
    Value::Map(pairs) => ...,
    other => vec![vec![other.clone()]],   // ← ArgList 走到这里！
};
```

当 `@each $item in $list` 中的 `$list` 是 `Value::ArgList` 时，
匹配到 `other` 通配符，把整个 ArgList 打包成**单元素** `[vec![ArgList]]`。

导致 `@each` 只迭代一次，`$item = ArgList([...])`，而非 ArgList 中的各元素。

### 触发链

```
getCssVarName($args...)              // 接收 rest-param → ArgList
  → joinVarName($args)               // $list = ArgList
  → @each $item in $list             // BUG: $item = 整个 ArgList
    → $name: $name + '-' + $item     // String + ArgList → Unsupported + operation
```

### 影响范围

element-plus 中有 **上千次** `joinVarName` 调用（每个 CSS 变量名都经过此函数）。
只要有 2 参数形式（`getCssVarName($name, $attribute)`），就会触发 ArgList 的 `@each` 失败。

## 解决方案

### 核心修复

在 `eval_each` 的 `match` 中，让 `ArgList` 与 `List` 共享相同迭代逻辑：

```rust
Value::List(es, _, _) | Value::ArgList(es, _, _) => {
    es.iter().map(|e| vec![e.clone()]).collect()
}
```

### 次要防御: add() 错误信息增强

`add()` 的 fallthrough 分支给更有意义的错误信息：
```
Cannot add {left_type} and {right_type}
```

## 文件变更

| 文件 | 变更 |
|------|------|
| `src/eval/control_flow.rs` | `@each` match arm 增加 `ArgList` 支持（核心修复） |
| `src/eval/value/ops.rs` | `add()` fallthrough 增强错误信息 |

## 验证

```bash
cargo test --test ep_full -- --nocapture
# 期望: ok=121 total=121 fail=0（+2 修复）
```

## spec 规范

- `@each` 迭代 `ArgList` 的行为必须与迭代 `List` 一致
- 单变量 `@each $item in $arglist` → $item 依次取各元素
- 多变量 `@each $a, $b in $arglist_of_pairs` → 按 pair 迭代（如有需要）
