# Selector Simplification — Design

## 问题

`@extend` 常产生冗余选择器：
- **Compound 重复**：`.a.a` 应简化为 `.a`
- **Superselector 冗余**：`.a.b` 被 `.a` 覆盖（superselector 关系 + specificity 条件满足时）

sass-spec `directives/use/extend/diamond/merge` 期望：
```
%in-other.a { x: y }
.a { @extend %in-other }   → .a.a → .a
.b { @extend %in-other }   → .a.b（被 .a 覆盖 → 消除）
=>
.a { x: y }
```

## 设计方案

新增 `src/css/selector_simplify.rs` 模块，集成到 `Serializer::serialize` 管线：

```
flatten_nodes → selector_simplify → merge_at_rules → serialize_expanded/compressed
```

### 阶段 1：Compound 内 Simple 去重
- 解析 selector 为 AST
- 每个 Compound 内部去重 simples（保持原有顺序）
- 序列化回字符串

### 阶段 2：Superselector 消除
- 按 `group_id` 分组（来自同一 eval_rule 输出的 rules）
- 仅比较相同 group 内且 declarations 相同的 rules
- 使用 `is_super` 模块的 `is_superselector` 做判断
- 移除被 superselector 覆盖的 redundant rule

### 关键约束
- **不跨 group 比较**：避免不同 eval 上下文的 rules 被错误消除
- **仅消除有 declarations 的 rules**：empty rules（可能有 children）保留
- **不修改 eval 阶段逻辑**：纯序列化阶段后处理

## 文件清单

| 文件 | 改动 |
|------|------|
| `src/css/selector_simplify.rs` | 新增 ~130 行 |
| `src/css/mod.rs` | 添加模块 + 调用点 |
| `src/eval/extend.rs` | 修复 `check_extend_targets` 单文件 placeholder 检测 |
| `CHANGELOG.md` | 添加 0.9.12 entry |

## 风险

- **误消除**：限制在相同 group_id + 相同 declarations 内比较，风险极低
- **性能**：O(n²) 仅在相同 group 内，通常 group 很小（2-5 个 rules）
- **兼容性**：如果 selector parser 解析失败，回退到原字符串（no-op）
