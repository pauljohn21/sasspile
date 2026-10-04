## Context

### 问题背景

EP (Element Plus) theme-chalk 使用 BEM mixin 模式（`b()` / `e()` / `m()` / `when()`）组织样式。核心模式：

```scss
@include b(checkbox) {
  &.el-checkbox--small {         // ← 复合选择器: .el-checkbox.el-checkbox--small
    @include e(input) {          // ← e() mixin 内部 @at-root { & { .el-checkbox__input { @content } } }
      @include when(indeterminate) { ... }
    }
  }
}
```

期望输出: `.el-checkbox.el-checkbox--small .el-checkbox__input{...}`

sasspile 当前输出: `.el-checkbox.el-checkbox--small .el-checkbox.el-checkbox--small .el-checkbox__input{...}`（重复前缀）

### 根因分析

`e()` mixin 展开为 `@at-root { #{$selector} { #{$currentSelector} { @content } } }`，其中:
- `$selector = &` (mixin 调用时刻的 env.current_selector)
- 当 `&.block--mod` 嵌套时，`&` = `.el-block.el-block--mod`（完整路径）

**问题核心**: `eval_at_root` 中 `&` 引用展开逻辑依赖 `env.get_selector()` 获取当前选择器，但:
1. 嵌套 `&.block--mod` 内部的 `&` 应该在规则求值时已被展开为完整路径
2. 当前实现中，当 `eval_at_root` 处理由 mixin 展开的 AtRoot 节点时，`env.get_selector()` 返回的是 mixin 保存时的选择器片段而非完整嵌套链
3. 导致: wrapper-skip 逻辑判断错误 → 外层 `nest_rule_in_children` 重复添加父前缀

### 当前代码位置

- `src/eval/mixin.rs::eval_at_root` (L424-503) — 处理 AtRoot 节点
- `src/eval/mixin.rs::exec_mixin` — 展开 mixin body
- `src/eval/rule.rs::nest_rule_in_children` (L400+) — 嵌套规则处理 (含 double-prefix 守卫)
- `src/eval/rule.rs::combine_selectors` — 选择器组合
- `src/eval/env.rs` / `env_impl.rs` — `current_selector`, `selector_chain` 字段

## Goals / Non-Goals

**Goals:**
- 修复 `e()` / `m()` / `when()` 在复合选择器块内部调用时的 `&` 展开
- 消除 selector doubling (.a--mod .a--mod) 和 selector missing (丢失 `.is-state` 前缀)
- 保持 sass-spec 无回归 (7877/12133)

**Non-Goals:**
- 不修复 EP Pipeline 产物差异 (lightningcss CSS vars, vendor prefix, mask vs border)
- 不修改 `calc()` / `color()` 等 CSS 函数
- 不修改 `@keyframes` / `@media` 等 at-rule 处理

## Decisions

### Decision 1: 在 `exec_mixin` 层传递完整 `&` 上下文

**选择**: 当 `exec_mixin` 展开 mixin body 时，如果 mixin 内部包含 `@at-root { & { ... } }` 或引用 `$selector`（= &），需要确保 body 内的 `&` 引用解析为调用时刻的完整父选择器链。

**实现**: 
- 在 `eval_at_root` 中，不再用 `env.get_selector()` 改为追踪从外层规则继承的 "inherited_selector_chain" 
- 通过 `env.get_selector_chain()` 获取完整嵌套链（已有此方法但未用于 @at-root 的父级展开）

**替代方案**: 
- A) 在 mixin 调用前 snapshot & 变量传入 → 侵入性强，需改 mixin 展开逻辑
- B) 在 AST 预处理阶段展开 `&` → 复杂度高，需处理变量插值

**理由**: 方案 A 最简洁且不侵入 AST 层。

### Decision 2: 增强 eval_at_root 的 AtRoot selector 构造

**选择**: 当 `selector` 参数存在（显式 at-root 路径）时，`parent_sel` 应反映**嵌套规则展开后的完整选择器**，而非 mixin 调用的片段。

**实现**:
```
// 当前（有 bug）:
let parent = env.get_selector().unwrap_or("").to_string();
let parent_sel = combine_selectors(&parent, &interpolated);

// 修复后:
let incoming_chain = env.get_selector_chain().map(String::from);
let parent = match incoming_chain {
    Some(chain) => chain,      // 完整嵌套链
    None => env.get_selector().map(String::from).unwrap_or_default(),
};
let parent_sel = match interpolated.contains('&') {
    true => combine_selectors(&parent, &interpolated),
    false => interpolated,
};
```

**风险**: 如果 `selector_chain` 在非嵌套 at-root 场景下被错误设置，可能破坏 `@at-root` 独立调用场景。需验证 `reset_selector` 正确清空 chain。

### Decision 3: 加强 nest_rule_in_children 的 double-prefix 守卫

**选择**: 在 `nest_rule_in_children` 中，子 selector 检测已增强前缀匹配（含 descendant 空格分隔符），但需在 `eval_rule` 的 post-processing 层也加入类似守卫。

**场景**: 当 AtRoot 输出被外层 Rule 的 body 处理时，当前 `nest_rule_in_children` 可能被调用于已经包含父链的 AtRoot 节点。

**实现**: 在 `nest_rule_in_children` 入口增加递归 guard，检测 parent 是否已经出现在 child selector 的开头（含 compound 和 descendant 两种模式）。

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|------|------|------|
| `selector_chain` 泄漏到非嵌套 @at-root | 可能改变 `@at-root .foo { ... }` 输出 | 在 `reset_selector` 创建新 env 时清空 chain |
| mixin 递归展开时 chain 重复累加 | `@include` 多层嵌套导致 chain 指数增长 | chain 应为路径快照而非累加器 |
| sass-spec @at-root 测试回归 | 破坏现有通过测试 | 运行 sass-spec 全量验证 |

## Migration Plan

1. **SPAN 插桩** (Phase 1): 在 `eval_at_root` / `nest_rule_in_children` 加 tracing span，记录 parent, chain, child_sel
2. **TRACE 采集**: 编译 checkbox.scss，收集 diff 位置的 trace
3. **根因定位**: 确认 trace 中哪个环节 parent chain 断裂
4. **实现修复**: 修改 `eval_at_root` 的 parent_sel 构造逻辑 + 加强 double-prefix 守卫
5. **验证**: cargo test + EP normalized + sass-spec (零回归)

## Open Questions

1. `selector_chain` 在 `exec_mixin` 展开时是否正确传递？需验证 mixin body 的 env 是否携带 chain
2. `@at-root` 嵌套在 `@at-root` 内时（when → e → when），chain 是否正确重置？
3. `wrapper-skip` 逻辑与新的 parent_sel 构造是否仍有冲突？
