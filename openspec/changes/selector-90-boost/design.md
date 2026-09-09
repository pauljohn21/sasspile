## Context

`selector-extend()` 函数在 sass-spec `core_functions/selector` 目录有 408 个 DIFF 失败。`selector-ast-rewrite` 已修复了命名空间数据模型（`Namespace` 枚举），但 `extend_complex` 构建逻辑仍有四类缺陷。需要对照 `failures_json` 的 expected/actual data 逐一修复。

## Goals / Non-Goals

**Goals:**
- 修复 extend_complex 四类算法缺陷，使 selector-extend 通过率从 54% 提升到 90%+
- 保留 `Namespace::Empty` 在 extend/unify 链中的序列化（`|name` 格式）
- 生成所有排列组合、消除 tail combinator 的重复 compound

**Non-Goals:**
- 不重写 extend 框架，只修复具体算法路径
- 不改动选择器解析层（parser）和 AST 数据结构

## Decisions

### Decision NO-OP 命名空间保留

**问题**：extend 无操作时返回原始 selector，但 `unify_extendee_list` 合并过程中丢失 `Namespace::Empty` 的序列化信息。

**方案**：在 `extend_selector` 的 no-op 分支直接返回 `selector.clone()`（已是），但需要确保 unify 过程中不修改原始 selector 的显示格式。问题实际在 unify 输出时丢失了空命名空间标记。

**修复**：确保 `unify_extendee_list` 保留原始 namespace 信息，或在 extend 输出时恢复空命名空间。

### Decision Format 多余选择器

**问题**：`extend/format` 场景生成 `c, e, d c, d e`（多了 `d e`）。

**根因**：扩展后的去重逻辑（`acc.contains(&c)`）使用结构相等比较，但 `d c` 和 `d e` 不是重复项——真正的问题是扩展不应生成 `d e`。需要在扩展算法中检查生成的每个 complex 是否确实由 extender 产生。

**修复**：在 `build_extended_complex` 的回溯路径生成时，增加"extender 部分必须真正出现"的校验。

### Decision Complex 统合缺漏

**问题**：`extend/complex/with_unification` 场景只生成 2 个结果中的 2/3。

**根因**：unify 生成的排列组合未全部传入 extend。`unify_extendee_list` 返回统合后的 extendee，但 extend 循环只处理单一统合结果，未考虑多种统合方式。

**修复**：遍历所有统合结果，为每个统合后的 extendee 执行 extend，合并结果并去重。

### Decision Tail Combinator 重复 Compound

**问题**：`extend/complex/trailing_combinator` 场景输出 `c.x .d .d`（`.d` 重复）。

**根因**：tail combinator 处理路径中，`remaining` 与 extender tail 的合并以及 suffix 连接逻辑有重复附加。

**修复**：在 tail combinator 分支的 suffix/merge 逻辑中消除重复 compound 附加。

## Risks / Trade-offs

- **[回归风险]** extend 是核心选择器操作，修改影响 `selector-extend()` 函数的所有路径。Mitigation: 用 failures_json 工具验证每次修改后 PASS 增加、无新增失败
- **[排列组合爆炸]** 修复"生成所有排列"可能产生指数级结果。Mitigation: 限制为 sass-spec 合理的排列数，保持去重
- **[命名空间语义]** `Namespace::Empty`（`|name`）和 `Namespace::None`（`name`）的边界条件。Mitigation: 对照 spec 逐一验证
