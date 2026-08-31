# Design: Nested Output Format + Quick Fixes

## Architecture

### 当前流程（展平模式）

```
AST → Evaluator → RuleBuilder（展平选择器） → CssNode（扁平） → flatten_nodes → serialize_expanded
                                     ↑ 问题根源
```

`RuleBuilder::push` 第 49 行：`combine_selectors(&self.selector, &child_sel)` 把 `a` + `b` 合并为 `a b`，子规则从 `children` 提取到顶层 `result`。

### 目标流程（保留嵌套）

```
AST → Evaluator → RuleBuilder（保留嵌套） → CssNode（嵌套） → merge_nested → serialize_expanded
                                                       ↓
                                          compressed: flatten_nodes → serialize_compressed
```

## Phase 2 详细设计：RuleBuilder 重构

### 核心改动

`RuleBuilder::push` 不再合并选择器，保留子规则在 `children` 中：

```rust
// 当前（展平）
CssNode::Rule { selector: child_sel, declarations: child_decls, children: child_kids } => {
    self.flush_decls();
    let combined = Evaluator::combine_selectors(&self.selector, &child_sel);
    if !child_decls.is_empty() {
        self.result.push(CssNode::Rule { selector: combined.clone(), declarations: child_decls, children: vec![] });
    }
    // 子规则也被展平到顶层
    for kid in child_kids { ... }
}

// 目标（保留嵌套）
CssNode::Rule { selector: child_sel, declarations: child_decls, children: child_kids } => {
    self.flush_decls();
    // 保留子规则在 children 中，不展平
    self.result.push(CssNode::Rule {
        selector: child_sel,  // 保持原样，不合并
        declarations: child_decls,
        children: child_kids,  // 保留嵌套
    });
}
```

### `&` 引用处理

`&` 替换仍在 `eval_rule` 中处理（求值阶段），不延迟到序列化：

```scss
a { &.b { c: d; } }
// eval_rule 中 & 被替换为 a，得到 selector = "a.b"
// RuleBuilder 保留为 children
// 输出：a { a.b { c: d; } }
```

但 dart-sass 输出 `a { &.b { c: d; } }` — 保留 `&`！

**决策**：`&` 替换延迟到序列化阶段。`eval_rule` 不替换 `&`，保留原始 `&` 选择器在 CssNode 中。

### @media 嵌套

```scss
a { @media (min-width: 100px) { b { c: d; } } }
```

当前：`@media` 内的 Rule 被展平为 `a b`。
目标：保留嵌套 `a { @media { b { c: d; } } }`。

`RuleBuilder::push` 的 `AtRule` 分支保持 `children` 不变即可。

## Phase 1 设计：快速修复

### 1. 字符串到数字隐式转换

```rust
// "0" → 0.0
fn coerce_number(v: &Value) -> Result<f64> {
    match v {
        Value::Number(n, _) => Ok(*n),
        Value::String(s, _) => s.parse::<f64>()
            .map_err(|_| SassError::Eval(format!("$number: {v} is not a number."))),
        _ => Err(SassError::Eval(format!("$number: {v} is not a number."))),
    }
}
```

### 2. 运算符支持

- `Calc + Calc` → 字符串拼接 `format!("{a} + {b}")`
- `Calc + Number` → `format!("{a} + {b}")`
- `String + Calc` → 字符串拼接

### 3. 参数验证

- `set-nth` 接受 3 参数（当前正确，但错误消息可能不匹配）
- `if` 接受 3 参数
- `rgba` 接受 3-4 参数
