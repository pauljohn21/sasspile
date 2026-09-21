## Why

经过 5 次不同位置的 `&` 展开尝试（Value::ParentRef / eval_interp_segments / eval_value::Interp / eval_variable），全部导致 avatar.scss 从 IDENTICAL 退步为 EmptySelector error。

**根本原因**：`sasspile` 的 `Env.current_selector: Option<String>` 是**扁平字符串**，无法区分 `&` 的「定义时父选择器」和「使用时的当前选择器」。当 `&` 存入变量后字面存储为 `"&"`，后续在 `@at-root`/`@content` 链中 `#{$var}` 展开时，`&` 被错误的上下文选择器替换。

这不是一个点的 bug，而是选择器上下文传播机制的系统性缺陷，需要架构级重构。

## What Changes

### 核心重构：SelectorFrame 栈替代扁平字符串

将 `Env.current_selector: Option<String>` 替换为 `Env.selector_frames: SelectorFrameStack`：

```rust
struct SelectorFrame {
    /// 当前层级的完整选择器（包含所有嵌套层）
    rendered: String,
    /// 父选择器帧的索引（栈中位置）
    parent_idx: Option<usize>,
    /// 此帧的 "原始选择器"（未 combine 的 flat selector）
    raw_selector: String,
    /// 此帧是否由 @at-root 创建
    is_at_root: bool,
}
```

**关键语义**：
- `.el-popper { $arrow-selector: #{& + '__arrow'}; }` 时，`Value::Interp` 中的 `&` 展开为 `.el-popper`（当前帧的 rendered）
- 变量 `$arrow-selector` 存储为 `.el-popper__arrow`（已展开的字面量）
- 后续 `> #{$arrow-selector}::before` 展开时不再有 `&`，避免二次展开

### BEM mixin 兼容性

`@mixin m($modifier)` 模式：
```scss
$selector: &;  // 存储为 Value::String("&")，不展开
$currentSelector: '';
@each $unit in $modifier {
  $currentSelector: #{$currentSelector + $selector + '--' + $unit + ','};
  // ↑ 在 Value::Interp 展开时，& 被展开为定义时父选择器
}
@at-root { #{$currentSelector} { @content; } }
```

关键：**只在 `Value::Interp` 内部展开 `&`**（不在 eval_variable、不在 literal string）。这样 `$selector: &` 存字面 `"&"`，但在 `#{...}` 表达式求值时根据当前上下文展开。

### @at-root 语义修正

`@at-root` 内的 `&` 展开应使用**调用者上下文**而非当前嵌套上下文。通过 `SelectorFrame.is_at_root` 标志实现：
- 正常嵌套：`&` 展开为栈顶帧的 rendered
- `@at-root` 内：`&` 展开为 `is_at_root=false` 的第一个祖先帧的 rendered

**BREAKING**: `Env` 结构体变更（`current_selector` 字段替换为 `selector_frames`），影响所有 `env.get_selector()` / `env.with_selector()` 调用点。

## Capabilities

### New Capabilities
- `selector-frame-stack`: 用栈式帧替代扁平字符串，支持正确的 `&` 定义时展开
- `at-root-context-propagation`: `@at-root` 内的 `&` 正确展开为调用者上下文

### Modified Capabilities
- `selector-ast`: 选择器 AST 需适配新的 SelectorFrame 查询 API
- `mixin-context-propagation`: mixin/@content 调用的选择器上下文传播
- `value-parent-ref`: 新增 `Value::ParentRef` 或等价机制表达表达式中的 `&`

## Impact

**受影响的文件**：
- `src/eval/env.rs` — Env 结构体重构
- `src/eval/env_impl.rs` — with_selector / get_selector 替换为 SelectorFrameStack API
- `src/parse/ast/mod.rs` — Value 新增 ParentRef 变体（可选，取决于最终设计）
- `src/parse/expr/literals.rs` — Amp token 返回 ParentRef
- `src/eval/value/mod.rs` — eval_value ParentRef 处理 + Value::Interp & 展开
- `src/eval/rule.rs` — eval_rule 使用新的选择器帧栈
- `src/eval/mixin.rs` — exec_mixin / eval_content 选择器上下文传播
- `src/eval/hoist.rs` — @at-root 提升逻辑适配帧栈
- 所有 `env.get_selector()` / `env.with_selector()` 调用点（约 30+ 处）

**风险**：架构级改动，影响所有选择器相关功能。需要全面回归测试。

**预期收益**：
- EP 一致性从 52/121 (43%) → 90+/121 (75%+)
- 修复所有 BEM mixin 相关的 `&` 展开问题
- 修复 `> img` / `> .el-popper__arrow` 等孤儿选择器问题

**不做的事**（Non-Goals）：
- 不改变 CSS selector AST（`SimpleSelector::ParentReference` 不变）
- 不改变 `combine_selectors` 的匹配逻辑（`"&".replace` 语义）
- 不引入新的 CLI 或 public API
