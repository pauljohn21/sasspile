# EP Consistency Boost — Design Decisions

## D1: `&` 父选择器在插值中的展开

### 问题本质

SCSS 中 `&` 在插值 `#{...}` 内的行为：
- `#{& + '-root'}` 中的 `&` 应被替换为父选择器的**字符串值**
- sasspile 原实现将 `&` 解析为字面字符串 `"&"`，导致 `& + '-root'` 求值为 `"&-root"`

### 修复策略

新增 `expand_amp_in_interp` 函数，只展开 `#{...}` 内部的 `&`：

```rust
fn expand_amp_in_interp(s: &str, parent: &str) -> String {
    // 跟踪 #{...} 嵌套深度
    // interp_depth > 0 时遇到的 & → 替换为 parent（带引号作为字符串字面量）
    // interp_depth == 0 时的 & 保持不变（由 combine_selectors 处理）
}
```

选择器求值改用 `eval_selector_str`：
1. 检测 `&` 和 `#{` 同时存在
2. 调用 `expand_amp_in_interp` 展开插值内 `&`
3. 委托 `eval_interp_str` 处理其他插值

**关键设计决策**：
- 用 `"` + parent + `"` 包裹父选择器，确保 `eval_simple_expr` 将其识别为字符串字面量
- 字面量 `&` 保留给 `combine_selectors` 处理，避免双重展开

### 示例

```
输入: #{& + '-root'} (父选择器 = ".el-overlay")
  → expand_amp_in_interp: #{" .el-overlay" + '-root'}
  → eval_interp_str 求值: ".el-overlay-root"
```

## D3: mixin @at-root 源码位置（AtRootDirect）

### 问题本质

EP BEM mixin（`e()`, `m()`）内部 `@at-root { ... }` 用于生成子选择器。
当 mixin 在父 rule body 的**嵌套规则之后**调用时（如 `&:hover` 之后调 `@include e(icon)`），
RuleBuilder 把 `@at-root` 原始节点统一存入 `root_nodes`，`build()` 在固定位置插入
（第一个 `.el-backtop` 之后），源码顺序被破坏。

### 决策：新增 `CssNode::AtRootDirect` 节点类型

在 `exec_mixin` 求值完成后，将 `CssNode::AtRoot(inner, _)` 替换为
`CssNode::AtRootDirect(Box::new(n))`（每个 inner 节点独立包装）。

`RuleBuilder::push` 新增 `AtRootDirect` 分支：`flush_decls()` 后直接
`self.result.push(*inner)`，**bypass combine_selectors**——已在 mixin 内展开最终选择器，
不需要再次与父选择器组合。

全链路适配：`flatten_nodes` / `serialize_{expanded,compressed}` / `hoist` /
`extend` / `meta_ops` 均新增 `AtRootDirect` match arm，保持等效序列化行为。

### sass-spec 影响

约 -84 case（7977 → 7893），集中在 mixin 内使用 `@at-root` 的测试。
trade-off 接受：EP 一致性收益 > sass-spec 少量回归。

## D4: 字面 `&` 选择器在 AtRootDirect 中的条件展开

### 问题本质

`when(disabled)` 等 mixin 生成含字面 `&` 的选择器（如 `&.disabled`）。
该选择器通过 exec_mixin 时包装为 AtRootDirect(Rule("&.disabled", [...]))。
原始 AtRootDirect 处理路径 `self.result.push(*inner)` 绕过 combine_selectors，
导致字面量 `&` 原样输出，违反 Sass 规范。

### 决策：条件性 combine_selectors

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

**语义正确性证明**：
- 含 `&`：`&` 是对父选择器的引用，通过 combine_selectors 组合生成完整选择器
  例：`.el-item-selected` + `&.disabled` → `.el-item-selected.is-disabled` ✓
- 不含 `&`：已是完整选择器（如 `e()` mixin 生成的 `.el-backtop__icon`），直接 push 保持 at-root 语义 ✓

### sass-spec 影响

0 case。该分支仅对字面量 `&` 触发，sass-spec 测试中的 `&` 在插值内已被
`expand_amp_in_interp` 处理，到达 RuleBuilder 时不含字面 `&`。

## D2: color.mix 输出格式

### 问题本质

dart-sass 对 `color.mix()` 结果输出 `rgb(r%, g%, b%)` 格式，sasspile 输出 `#hex` 格式。

### 修复策略

在 `builtin_mix_modern` 中强制使用 `ColorOutput::RgbPercent`：

```rust
let mixed = Color::with_space(
    mixed_space, [r, g, bl], alpha,
    ColorOutput::RgbPercent,  // EP FIX
    lerped,
);
```

同时确保 downstream 路径（legacy 空间回退）保留 `RgbPercent` 输出模式：

```rust
None if a.space.is_legacy() => {
    Ok(Value::Color(Color::with_rgb(
        ..., mixed.output  // 保留 RgbPercent
    )))
}
```

## D3: @at-root RuleBuilder 排序

### 问题本质

dart-sass 将 `@at-root` 提升的节点放在**父声明块之后、嵌套子规则之前**。sasspile 原实现将 `root_nodes` 追加到末尾。

### 修复策略

修改 `RuleBuilder::build`：
1. 找到父声明块（selector == self.selector 的 Rule 节点）
2. 在父声明块后插入 `root_nodes`
3. 如果父声明块不存在（纯 mixin 调用），将 `root_nodes` 放在所有子节点之前

```rust
let mut combined: Vec<CssNode> = Vec::new();
let mut hoisted = false;
for node in self.result {
    if !hoisted {
        if let CssNode::Rule { selector, .. } = &node {
            if selector == &self.selector {
                combined.push(node);
                combined.extend(self.root_nodes.iter().cloned());
                hoisted = true;
                continue;
            }
        }
    }
    combined.push(node);
}
```

## D4: eval_at_rule 参数求值前移

### 问题本质

`eval_at_rule` 先用 `eval_nodes` 处理 body（触发 `Rule` 的 `enter_scope/exit_scope`），导致 mixin 局部变量丢失。后续 `eval_interp_str` 无法求值 `map.get($map, $key)`。

### 修复策略

将参数求值移到 body 处理之前，使用原始 `env`：

```rust
let params = if params.contains("#{") {
    eval_interp_str(&params, &env)  // 使用原始 env
} else {
    params
};
let (css, new_env) = Self::eval_nodes(body, env)?;  // 然后处理 body
```

## D5: calc() 内部 Sass 函数通用求值

### 问题本质

`calc(getCssVar("index","normal") - 1)` 中的 `getCssVar` 未被识别为函数调用。

### 修复策略

新增 `try_eval_calc_inner_functions()`：
1. 扫描 `calc()` 字符串中的 `ident(...)` 模式
2. 识别用户函数（env.get_function）
3. 求值并替换为返回值
4. 返回新的 Calc 字符串

## 修改文件汇总

| 文件 | 改动类别 |
|------|----------|
| `src/eval/rule.rs` | RuleBuilder AtRoot 排序 + eval_rule 选择器求值 |
| `src/eval/value/display.rs` | eval_selector_str + expand_amp_in_interp |
| `src/eval/value/mod.rs` | 导出 eval_selector_str |
| `src/eval/color.rs` | color.mix RgbPercent 输出 |
| `src/eval/at_params.rs` | 参数求值前移 |
