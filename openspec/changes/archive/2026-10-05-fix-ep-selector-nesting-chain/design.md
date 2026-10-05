# Design

## Context

EP 的 SCSS 源码大量使用 `#{& + '-suffix'}` 插值模式在嵌套规则中生成派生选择器。当前 `selector_combine.rs` 中 `starts_with_compound_prefix` 使用 `b".:#[>+~_-"` 作为分隔符集合——其中单 `-` 和单 `_` 导致 `.el-popper-selfdefine` 被错误判定为 `.el-popper` 的 compound 变体，从而跳过 descendant combine。

## Goals / Non-Goals

**Goals:**
- 修复 `starts_with_compound_prefix`：仅 `--` 和 `__` 触发 compound 判定
- 确保 14 个 EP 选择器嵌套失败文件通过
- 不破坏 sass-spec 中合法 BEM 模式

**Non-Goals:**
- 不修改 `eval_rule` 核心逻辑（仅调纯正函数）
- 不处理 `user-select:none` 丢失问题（独立根因）
- 不处理 `var()` fallback / calc 简化（独立变更）

## Decisions

### Decision 1: 分隔符集合精确化

**方案**: 将 `starts_with_compound_prefix` 的分隔符从 `b".:#[>+~_-"` 改为仅识别 `--` 和 `__` 作为 BEM 分隔符， `.` `:` `#` `[` `>` `+` `~` 保留为 structural 分隔符。

**Alternatives:**
1. 使用 `--` / `__` 双字符精确匹配 → 选此方案
2. 引入 "interpolation flag" 标记 `eval_selector_str` 产出的 selector → 侵入 CssNode 设计
3. 在 RuleBuilder 增加 heuristic "child 与 parent 长度差 <= 5 则不 compound" → 不可靠

**Rationale**: 双字符匹配最简单且不破坏现有 structural 检测。即使 sass-spec 有 `&-large` 单划线的场景，正确输出应为 `.btn-large`（pattern 通过 `eval_rule` 的 `&` 路径 + `combine_selectors` 已正确处理，不依赖 `starts_with_compound_prefix`）——该函数仅在 RuleBuilder 收到已展开的子 Rule 时才用于判断"是否已含前缀"。

### Decision 2: 不引入 CssNode 标记

interpolation 路径经过 `eval_selector_str` 展开后，selector 字符串已不含 `&`。理论上无法区分 "来自 interpolation" 和 "字面 selector"。但 Decision 1 的精确分隔符方案使得两种场景都正确——interpolation 场景的 child 不会再被误判为 compound。

## Risks / Trade-offs

- [Risk] sass-spec 含 `&-large` 嵌套测试（非 BEM 的 compound）若存在，可能因 separator 精确化改变行为 → Mitigation: 实现后运行 `cargo test --test sass_spec_full` 对比 pass 数
- [Risk] EP 可能还有其他选择器嵌套失败未被 classify 捕获 → Mitigation: 修复后重新运行 `ep_classify_test` 确认

## Migration Plan

无迁移需求——bug fix 不影响公开 API 或持久化格式。

## Open Questions

- Q: sass-spec 是否有 `&-large` 单 `-` suffix 嵌套模式的测试？ → 实现后统计确认
