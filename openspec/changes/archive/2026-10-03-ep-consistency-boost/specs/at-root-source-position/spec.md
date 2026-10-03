# Spec: AtRootDirect — Mixin @at-root 源码位置

## 背景

EP BEM mixin（`e()`, `m()`）内部使用 `@at-root { ... }` 生成子选择器。
当 mixin 在父 rule body 的**嵌套规则之后**调用时（如 `&:hover` 之后调 `@include e(icon)`），
RuleBuilder 原实现将 `@at-root` 节点存入 `root_nodes`，`build()` 在固定位置插入，导致输出顺序与源码相反。

## 需求

### REQ-1: 新增 CssNode::AtRootDirect 变体

```rust
pub enum CssNode {
    // ... 现有变体 ...
    /// 来自 mixin at-rule 的 @at-root 节点——已展开最终选择器，
    /// 在 RuleBuilder 中以源码位置处理，不参与 combine_selectors。
    AtRootDirect(Box<CssNode>),
}
```

### REQ-2: exec_mixin 后处理替换

mixin body 求值完成后，将 `CssNode::AtRoot(inner, _)` 替换为
`CssNode::AtRootDirect(Box::new(n))`（每个 inner 节点独立包装）。

```rust
let css: Vec<CssNode> = css
    .into_iter()
    .flat_map(|node| match node {
        CssNode::AtRoot(inner, _) => inner
            .into_iter()
            .map(|n| CssNode::AtRootDirect(Box::new(n)))
            .collect(),
        other => vec![other],
    })
    .collect();
```

### REQ-3: RuleBuilder::push 处理

```rust
CssNode::AtRootDirect(inner) => {
    self.flush_decls();
    // 条件分支见 spec/literal-ampersand
    self.result.push(*inner);
}
```

## 影响范围

- `src/css/node.rs` — CssNode 枚举 + fmt 实现
- `src/css/serialize.rs` — flatten_nodes 新增 AtRootDirect 分支
- `src/css/serialize_write.rs` — serialize_expanded/compressed 新增分支
- `src/eval/mixin.rs` — exec_mixin 后处理
- `src/eval/rule.rs` — RuleBuilder::push
- `src/eval/hoist.rs` — hoist_recursive
- `src/eval/extend.rs` — apply_extends
- `src/eval/meta_ops.rs` — collect_css
