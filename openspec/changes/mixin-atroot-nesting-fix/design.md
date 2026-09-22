## Context

### 当前状态

`exec_mixin` 处理 `@at-root` mixin 时，将 `AtRoot` 节点转换为多个独立的 `AtRootDirect` 兄弟节点：

```rust
// 当前实现（src/eval/mixin.rs exec_mixin）
CssNode::AtRoot(inner, _) => inner
    .into_iter()
    .flat_map(|n| match n {
        CssNode::Declaration { .. } => vec![n],
        other => vec![CssNode::AtRootDirect(Box::new(other))],
    })
    .collect(),
```

**问题**：当 `@mixin m(large)` 的 `@content` 包含 `@include e(header)` 时，`eval_rule("&!--large,")` 内部产生的子节点（来自 `e()`）被扁平化为与 `&!--large," 同级的 `AtRootDirect` 兄弟节点。

**Trace 证据**：
```
exec_mixin output n_params=1 n_css=6:
  Dir[Rule { selector: "&--large,", declarations: [font-size], children: [] }]
  Dir[Rule { selector: ".el-descriptions__header,", declarations: [...], children: [] }]
  ...
```

父节点 `children: []`，6 个独立兄弟节点——嵌套层级完全丢失。

### 约束

- 必须保留 EP BEM mixin 链的源码位置排序（AtRootDirect 的设计初衷）
- 不能破坏现有 73/121 EP IDENTICAL 文件
- 单文件 ≤ 500 行
- 纯 Rust 所有权模型，禁止 GC 依赖

## Goals / Non-Goals

**Goals:**
- 嵌套 `@at-root` mixin 内部子节点继承外层选择器上下文
- `AtRootDirect` 保留完整的子规则树结构
- `RuleBuilder::push` 递归组合子节点选择器

**Non-Goals:**
- 不改变非嵌套 `@at-root` 的行为
- 不重构整个 CssNode 类型系统
- 不修改 `flatten_nodes` 或 `serialize` 层

## Decisions

### Decision 1: 保留 AtRootDirect 嵌套结构

**选择**：修改 `exec_mixin` 的 `AtRoot` 转换逻辑，将整个 `Rule` 节点（含 children）包裹在单个 `AtRootDirect` 中，而非将 children 扁平化。

**替代方案**：
- A. 在 `RuleBuilder::push` 中通过 `&` 标记回溯重建嵌套 → 不可行，扁平化后无父子关系
- B. 使用新的 `CssNode::AtRootNested` 类型 → 增加类型复杂度，现有 `flatten_nodes` 需适配
- C. 在 exec_mixin 中保留原始 `AtRoot` 结构不转换 → 破坏 EP BEM 排序修复

**理由**：方案 A 最小改动，保留 `AtRootDirect` 类型语义，仅修改转换逻辑从"扁平化"变为"保留嵌套"。

### Decision 2: RuleBuilder::push 递归组合子节点

**选择**：在 `AtRootDirect` 分支中，当 selector 含 `&` 时，组合外层选择器后递归处理 children。

```rust
if selector.contains('&') {
    let combined = Evaluator::combine_selectors(&self.selector, &selector);
    let nested_children = children
        .into_iter()
        .map(|c| Self::combine_atroot_child(&combined, c))
        .collect();
    self.result.push(CssNode::Rule { selector: combined, declarations, children: nested_children });
}
```

**辅助函数** `combine_atroot_child`：
```rust
fn combine_atroot_child(parent_sel: &str, child: CssNode) -> CssNode {
    match child {
        CssNode::Rule { selector, declarations, children } => {
            let combined = format!("{parent_sel} {selector}");
            let nested_children = children.into_iter()
                .map(|c| Self::combine_atroot_child(&combined, c))
                .collect();
            CssNode::Rule { selector: combined, declarations, children: nested_children }
        }
        other => other,
    }
}
```

**理由**：递归组合确保任意深度的嵌套 `@at-root` 都能正确继承选择器上下文。

### Decision 3: Declaration 节点保持原样

**选择**：`AtRoot` 内部的 `Declaration` 节点不包裹在 `AtRootDirect` 中，直接作为 `CssNode::Declaration` 返回。

**理由**：`Declaration` 无选择器，不需要 `@at-root` 位置标记。包裹在 `AtRootDirect` 中会导致 `flatten_nodes` 将其展开为裸声明（无效 CSS）。

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| 保留嵌套结构可能改变 EP 已通过的 73/121 文件输出 | 运行 `ep_normalized_test` 全量验证，对比前后 DIFF |
| 递归组合可能产生重复选择器前缀 | `combine_atroot_child` 仅在 `AtRootDirect` 含 `&` 分支触发，正常嵌套已由 RuleBuilder 处理 |
| 深层嵌套可能导致栈溢出 | EP 实际嵌套深度 ≤ 3（b→m→e），递归安全 |
| `format!("{parent_sel} {selector}")` 可能产生多余空格 | `sanitize_selector` 清理尾随空格和逗号 |

## Migration Plan

1. **Phase 1**: 修改 `exec_mixin` 保留嵌套结构
2. **Phase 2**: 修改 `RuleBuilder::push` 递归组合子节点
3. **Phase 3**: 运行 `ep_normalized_test` 验证
4. **Phase 4**: 运行 `SPEC_STORE_CMD=run` 验证 sass-spec 无回归
5. **Phase 5**: 运行核心测试 202/202

**回滚策略**：git revert 单次 commit，恢复原始扁平化逻辑。

## Open Questions

- `combine_atroot_child` 是否应处理 `AtRootDirect` 子节点（理论上不应出现，但防御性处理）？
- 是否需要为 `descriptions-item.scss` 等文件添加专项测试用例？
