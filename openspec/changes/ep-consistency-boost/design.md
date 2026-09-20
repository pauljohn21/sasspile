# EP Consistency Boost — Design Decisions

## D1: `&` 父选择器语义（Category A）

### 问题本质

BEM `m()` mixin 的实现：
```scss
@mixin m($modifier) {
  $selector: &;
  $currentSelector: '';
  @each $unit in $modifier {
    $currentSelector: #{$currentSelector + $selector + $modifier-separator + $unit + ','};
  }
  @at-root {
    #{$currentSelector} { @content; }
  }
}
```

`&` 在选择器中的语义是"当前嵌套上下文的父选择器"。`@at-root` 将规则提升到文档根级，但 `&` 仍需展开为调用者上下文的父选择器。

### 失败路径分析

sasspile 当前对 `@at-root` 的处理（`RuleBuilder::push`）：
```
CssNode::AtRoot(nodes, query) => {
    match without_media || without_supports || without_all || with_rule {
        true => { nest_rule_in_children(&self.selector, nodes) }
        false => { self.root_nodes.extend(nodes); }  // 问题在这里！
    }
}
```

当 `query` 为空（默认 `@at-root`），走 `false` 分支：直接将子节点追加到 `root_nodes`，绕过父选择器组合。即使子节点选择器中包含 `&`，也不会被展开。

### 修复策略

修改 `RuleBuilder::push` 的 `@at-root` 分支：对无 query 的 `@at-root`，检查子节点选择器是否包含 `&`。如果包含，说明这些选择器需要父选择器展开，走 `nest_rule_in_children`；否则走原始 `root_nodes.extend`。

```
let needs_parent_expansion = nodes.iter().any(|n| selector_contains_ampersand(n));
match (without_media || without_supports || without_all || with_rule || needs_parent_expansion) {
    true => { nest_rule_in_children(&self.selector, nodes) }
    false => { self.root_nodes.extend(nodes); }
}
```

`nest_rule_in_children` 内部已调用 `combine_selectors(parent, child)`，其中 `child.replace('&', p)` 会正确展开。

### `&` 检测规则

需要递归检测 `CssNode::Rule.selector` 是否包含字面 `&` 字符。注意：仅选择器字符串中的 `&` 需要展开，declarations 或 at-rule params 中的 `&` 不需要。

## D2: SCSS 嵌套函数求值（Category C）

### 问题本质

`sass:string.unquote()` 和 `map.get()` 是 sass:map / sass:string 模块的函数，已注册为 builtin。问题在于调用链：
```scss
@mixin res($key, $map: $breakpoints) {
  @media only screen and #{string.unquote(map.get($map, $key))} { @content; }
}
```

`string.unquote(map.get(...))` 是嵌套 builtin 调用。sasspile 需要支持 builtin 函数的嵌套组合表达式求值。

### `getCssVar` 嵌套

EP 定义了自定义 `@function getCssVar($args...)`（在 `mixins/function.scss`）。当表达式为 `calc(getCssVar("x") - 4px)` 时，`getCssVar` 应被调用返回 `var(--el-xxx)`，然后外层 `calc()` 组合。这需要表达式求值器正确识别 `getCssVar(...)` 作为函数调用而非字面文本。

### 修复策略

1. `string.unquote(map.get(...))` 确保 builtin 嵌套调用被正确求值
2. `getCssVar` 在插值或属性值表达式中确认 dispatcher 能识别 `@function` 定义的函数

## D3: CSS 输出格式对齐（Category B）

### B1: 颜色常量

`white`、`black` 是 CSS 命名颜色。dart-sass 输出的字节映射为 hex。规范化方案：序列化时检测常见命名色（white/black/red/transparent 等）并映射为 dart-sass 的 hex 输出。

### B2: CSS 函数名大小写

`scalex` → `scaleX`, `translatex` → `translateX`。这是 CSS 标准函数名规范大小写。sasspile 可能在序列化时保留了输入的小写形式。需在 serialize 阶段做函数名规范化。

### B3: rgba → transparent

`rgba(0, 0, 0, 0)` 是 `transparent` 的等价表示。dart-sass 规范化为 `transparent`。可在颜色规范化阶段检测全零 alpha 并转换。

### B4: Keyframes 百分号转义

sasspile 序列化 `@keyframes` 时将 `%` 转义为 `\%`，dart-sass 不转义。需要在 keyframes 子规则的序列化中取消 `\%` 转义。

## 修改文件预估

| 文件 | 改动类别 |
|------|----------|
| `src/eval/rule.rs` | RuleBuilder::push AtRoot 分支 |
| `src/css/serialize_write.rs` | 颜色/函数名/百分号格式化 |
| `src/eval/builtin/dispatch.rs` | 嵌套 builtin 调用 |
| `src/eval/value/mod.rs` | getCssVar 在表达式中的识别 |
