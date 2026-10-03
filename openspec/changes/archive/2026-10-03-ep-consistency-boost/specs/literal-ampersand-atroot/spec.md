# Spec: 字面 `&` 选择器在 AtRootDirect 中的条件展开

## 背景

`when(disabled)` 等 mixin 生成含字面 `&` 的选择器（如 `&.disabled`）。
该选择器经过 AtRootDirect 包装后，原 `self.result.push(*inner)` 路径绕过了
combine_selectors，导致字面量 `&` 按原样输出，违反 Sass 规范。

## 需求

### REQ-1: RuleBuilder::push AtRootDirect 分支增强

```rust
CssNode::AtRootDirect(inner) => {
    self.flush_decls();
    if let CssNode::Rule { selector, declarations, children } = *inner {
        match selector.contains('&') {
            true => {
                let combined = Evaluator::combine_selectors(&self.selector, &selector);
                self.result.push(CssNode::Rule { selector: combined, declarations, children });
            }
            false => {
                self.result.push(CssNode::Rule { selector, declarations, children });
            }
        }
    } else {
        self.result.push(*inner);
    }
}
```

### REQ-2: 语义正确性

- **含 `&` 分支**：`&` 是对父选择器的引用，结合 RuleBuilder 当前 selector 展开。
  例：parent=`.el-segmented__item-selected`, child=`&.disabled`
  → `.el-segmented__item-selected.is-disabled` ✓
- **不含 `&` 分支**：已是完整选择器（如 `e()` mixin 生成的 `.el-backtop__icon`），
  直接 push 保持 at-root 提升语义 ✓

### REQ-3: 测试覆盖

- `tests/test_ep_amp_fix.rs` — 5 个 EP 文件验证无字面 `&.` 输出
- 核心测试 130/130 通过
- sass-spec 无新回归

## 影响范围

- `src/eval/rule.rs` — RuleBuilder::push AtRootDirect 分支
