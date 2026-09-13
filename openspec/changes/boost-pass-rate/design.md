## Context

sass-spec 当前通过率 ~7677/12131 (63.3%)。通过诊断工具识别出三个层次的失败：

1. **Quick Wins**：@extend (53%, 12 fails)、meta (63%, 179 fails)、math 边界 (86%, 66 fails) — 每个修复可带来 10-50 cases 收益
2. **Root Cause**：`+` 运算符 catch-all 报错 "Unsupported + operation"（影响 EP 编译和 sass-spec）、CSS 序列化格式不一致（413 fails 中约 30-40% 是格式问题）
3. **架构债务**：selector AST 操作不完整（45% 通过率，489 fails 最大单区域失败池）

当前 `src/eval/value/ops.rs` 的 `add()` 函数已实现 Number+Number、String+String、String+Number、String+Color、String+Null、String+Calc、Calc+Calc、Number+Calc、List+List、List+Other 等组合，但缺失：
- Map + Map（Map 合并）
- Map + Null / Null + Map
- Bool + Bool（应转为字符串拼接）
- Bracketed_List + Something（当前 non-bracketed List + other 有但不区分括号）

## Goals / Non-Goals

**Goals:**
- Phase 1 修复带来 +90 cases（63.3% → 64.1%）
- Phase 2 修复带来 +180 cases（64.1% → 65.6%）
- Phase 3 selector 完善带来 +200 cases（65.6% → 67.3%）
- 不引入回归（核心测试 202/202 维持）

**Non-Goals:**
- 颜色系统全面修复（已跳过策略）
- dual-ast 架构重构（另一个独立变更）
- dart-sass 行为对齐（正确性以 sass-spec 为准）

## Decisions

### 1. + 运算符补全策略

**决策**：在 `src/eval/value/ops.rs::add()` 中按 variant 优先级依次添加缺失组合，而非重构为泛型 trait。

**理由**：
- `add()` 已有清晰 match 结构，新增 arm 无需改动现有逻辑
- Rust enum 穷尽性检查天然保证不遗漏新 variant
- trait 方案需要为每个 variant 实现 trait，ROI 太低

**替代方案**：trait-based `Addable` — 过度设计，当前不需要扩展点。

### 2. CSS 序列化修复顺序

**决策**：按 CSS 结构层级修复 — 先 `@rule` 后 selector 后 declaration。

**理由**：@rule 格式错误（`@media` `@keyframes` `@font-face`）影响范围大且修复集中，declaration 值差异多但每个只影响单个 case。

**风险**：declaration 值格式化可能涉及浮点精度、颜色格式等多种因素，难以一次性覆盖所有场景。

### 3. Selector AST 操作实现

**决策**：在现有 `selector_unify.rs / selector_is_super.rs / selector_extend.rs` 基础上扩展 `selector_nest.rs` 和 `selector_append.rs`。

**理由**：已有三个模块奠定了拆分模式，延续现有架构风格。直接在 `selector_extend.rs` 中完善 extend 语义可选但为保持文件 ≤500 行而新建模块。

**替代方案**：合并回单一 `selector_ops.rs` — 违反 ≤500 行规则。

### 4. meta 内省修复

**决策**：`get-function` 返回 `Value::Function` 引用（含 name + arity），`keywords($args)` 返回 Map 而非 List。

**理由**：sass-spec 期望 `get-function` 返回可调用的 Function 值，当前返回 String 名导致后续调用失败。`keywords()` 作为 `@function` 内辅助，必须返回 Map 才能 `map-get` 使用。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| CSS 序列化改动影响大量 case | 分批提交，每批后跑 `test_sass_spec_full_stats` 验证无回归 |
| + 运算符新增组合可能与 libsass 行为不一致 | 以 sass-spec expected output 为准，不参考 libsass |
| selector extend 修复引入循环引用 | 复用现有的 `seen_extends: HashSet<String>` 检测 |
| math 边界修复可能影响现有 number 精度 | 仅处理 NaN/Infinity/特殊输入，不修改正常数值路径 |

## Open Questions

1. `Map + Map` 合并时键冲突如何处理？（后值覆盖 vs 报错 — 倾向与 libsass/Sass 语义一致，后值覆盖）
2. `get-function` 是否需要处理内置函数？（当前内建函数无 Sass 可调用的 Function value）
3. CSS `@container` 规则是否已在序列化层支持？
