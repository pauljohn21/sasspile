# Design — EP Rule Refactor & Bugfix

## Context

See `proposal.md` — Why for motivation and What Changes for scope. Current `rule.rs` (714 lines) mixes pure functions, state machine, and `Evaluator` impl — with 3 duplicated prefix-detection blocks that cause 16 EP DIFF files (selector doubling & nested-prefix bugs).

## Goals / Non-Goals

**Goals:**
- 拆分为 3 个职责清晰的模块，全部 ≤ 350 行
- 消除 3 处 `starts_with_compound_prefix` 重复 → 1 处 `has_descendant_prefix`
- 修复 Category A (6 files 选择器加倍) 和 Category B-C (10 files 嵌套结构)
- 函数式风格：纯函数集合独立到 `selector_combine.rs`，零 `&mut` 参数

**Non-Goals:**
- 不修改 `eval_rule` 入口语义（子作用域 enter/exit 逻辑不变）
- 不修改 `build()` 的 @at-root 提升算法
- 不修改 EP 管线产物（autoprefixer/lightningcss）处理策略
- 不改 mixin.rs 架构（分次处理）
- 不新增 sass-spec capability（现有 spec 已"要求"正确行为）

## Decisions

### Decision 1: 文件拆分方案

```
selector_combine.rs (~180 lines)  ← 纯函数，无状态
├── split_selectors_respecting_parens()
├── combine_selectors()           ← 核心：parent + child → 组合选择器
├── starts_with_compound_prefix() ← 保留（向后兼容）
├── has_descendant_prefix()       ← 新增：统一前缀检测（compound + descendant）
└── resolve_ampersand_in_nodes()  ← 递归 & 解析

rule_builder.rs (~250 lines)      ← 状态机，消费纯函数
├── RuleBuilder 结构
├── new() / flush_decls()
├── push()                        ← 3 处 combine 统一为 has_descendant_prefix
├── push_atroot_direct()          ← 同上
└── build()

rule.rs (~300 lines)              ← Evaluator 方法
├── eval_rule()                   ← 入口（调整内部引用）
└── selector_contains_ampersand()
```

**Alternatives considered:**
- A) 仅新增 `has_descendant_prefix` 不改结构 → 不能解决超标问题
- B) 拆为 2 files（rule.rs + selector_combine.rs）→ RuleBuilder 仍在 rule.rs，仍在 500+ 行

### Decision 2: `has_descendant_prefix` 设计

```rust
/// 检测 child selector 是否已包含 parent 前缀（compound 或 descendant 两种模式）。
/// 替代 3 处分散的 starts_with_compound_prefix 调用。
fn has_descendant_prefix(parent: &str, child: &str) -> bool {
    if parent.is_empty() || child == parent {
        return !parent.is_empty();
    }
    let separators: &[u8] = b".:#[>+~";
    // Compound prefix: .parent.child 形式
    let compound = child.len() > parent.len()
        && child.starts_with(parent)
        && child.as_bytes().get(parent.len()).is_some_and(|c| separators.contains(c));
    // Descendant prefix: .parent .child 形式（中间有空格）
    let descendant = child.starts_with(&format!("{parent} "));
    compound || descendant
}
```

**当前 `starts_with_compound_prefix` 的缺陷：**
- `parent=".el-alert"`, `child=".el-alert .el-alert__icon"` → 返回 `false`（有空格但不在末尾）
- `parent=".el-input-otp.is-disabled"`, `child=".el-input-otp.is-disabled,.el-input-otp__input"` → 逗号后不检测

**Alternatives:**
- 用正则 → 引入 regex crate，增加复杂度；纯字节检查更高效

### Decision 3: 新模块可见性

- `selector_combine` → `pub(crate)` 模块（eval 内部使用）
- `rule_builder` → `pub(crate)` 模块
- `mod.rs` 重导出：保持 `eval::rule_builder::RuleBuilder` 可用

### Decision 4: 测试策略

纯函数独立后可单独单元测试：
- `tests/selector_combine_test.rs` — 覆盖 `combine_selectors`、`has_descendant_prefix`
- 现有 `compile_test` 43 + `stage_test` 8 包含选择器场景，作为回归 baseline

## Risks / Trade-offs

| Risk | 概率 | 缓解 |
|------|------|------|
| 拆分导致内部 API 改名影响 mixin.rs | 中 | 用 `pub(crate)` + `mod.rs` 重导出保持接口稳定 |
| `has_descendant_prefix` 误报（合法加倍被当已有前缀） | 低 | 单元测试覆盖 edge case；sass-spec 全量作为回归 |
| sass-spec 选择器相关用例退化 | 低 | 纯函数拆分是等价变换；新增单元测试覆盖 |

## Migration Plan

**实施顺序**（保证每步可独立编译运行）：

1. 新建 `selector_combine.rs`，移动纯函数，原 `rule.rs` 用 `use` 导入 → 验证编译 + 核心测试
2. 新建 `rule_builder.rs`，移动 `RuleBuilder` + impl → 验证编译 + 核心测试
3. 精简 `rule.rs` 为 `Evaluator` impl → 验证编译 + 核心测试
4. 添加 `has_descendant_prefix` 替代 `starts_with_compound_prefix` → 验证测试 + EP classify
5. 添加单元测试 → 验证 edge case
6. 全量 sass-spec 验证无退化

**Rollback**：每步独立 commit，可单步 revert。

## Open Questions

- 无（设计决策已完整）
