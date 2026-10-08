# Design

## Context

See proposal.md - Why. 当前覆盖率 1.25%（67/5342 行匹配）。6 个根因横跨序列化、解析器、求值器三个模块，需协调修复顺序以最大化渐进覆盖率提升。

## Goals / Non-Goals

**Goals:**
- 修复序列化格式（B6/B7）→ coverage 1.25% → ~50%
- 修复 `!important`（B8）→ coverage ~50% → ~62%
- 修复厂家前缀 + mixin 展开（B9/B11）→ coverage ~62% → ≥ 99%
- 修复 Sass Maps 格式（B10）→ 消除特定 decl 乱码

**Non-Goals:**
- 不改变公共 API（CompileBuilder/from_string/from_path）
- 不引入新依赖
- 不修改 Bootstrap submodule 或测试 fixtures

## Decisions

### Decision 1: 修复顺序 — 格式优先，语义次之

**选择**: B6/B7（格式）→ B8（!important）→ B9-B11（语义/mixin）

**理由**: 格式修复影响所有输出行，是最大杠杆点；!important 是独立功能修复；厂家前缀和 mixin 展开涉及复杂的调用链分析，需要更多调试时间。渐进修复允许每步验证覆盖率提升。

### Decision 2: `!important` 表示 — 结构体字段

**选择**: 在 `StyleDecl` 结构体新增 `important: bool` 字段。Parser 遇到 `!Token` 时标记，Serializer 在输出时追加 ` !important`。

**替代方案**:
- (A) 将 `!important` 作为 value 的一部分（String 拼接） — 被否决，value 解析应与标记分离
- (B) 新增 `ImportantDecl` variant — 过度设计

**理由**: bool 字段最小侵入，符合现有 StyleDecl 结构。

### Decision 3: 缩进修复 — depth 语义校正

**选择**: 修改 `format_expanded` 的 depth 起始值为 1（根节点），每层 +1。缩进量 = `(depth - 1) * 2` spaces。

**理由**: depth=1 对应顶层规则内 decl，应缩进 2 空格（1 层 × 2）；depth=2 对应嵌套规则内 decl，应缩进 4 空格（2 层 × 2）。当前实现深度偏移 1 导致过量缩进。

### Decision 4: 选择器空格 — 格式化层修复

**选择**: 在 `format_expanded` 中选择器输出时统一插入 ` {`（空格+花括号），而非在 parser/eval 层处理。

**理由**: 格式化层是纯输出逻辑的修改点，不影响 AST 语义。Compressed 模式不受影响（无空格）。

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| 缩进修复影响现有单元测试 | 修复后需更新 integration_test.rs 中的 expected 字符串（预期为正向改善） |
| !important 解析影响现有 decl | 仅当 value 后有 `!` 标记才触发，向后兼容 |
| mixin 展开引入循环 | 依赖现有 @include 安全阀（MAX_INCLUDE_DEPTH） |

## Migration Plan

1. **Step 1**: 修复 B6/B7（序列化）→ 运行 bootstrap_dist_check 验证 coverage ~50%
2. **Step 2**: 修复 B8（!important）→ 验证 coverage ~62%
3. **Step 3**: 修复 B9/B11（前缀+mixin）→ 验证 coverage ≥ 99%
4. **Step 4**: 修复 B10（Maps 格式）→ 最终清理

每个 Step 独立 commit，可在任意点回滚。

## Open Questions

- [Q1] `!important` 是否可能出现在嵌套 property 如 `margin: { top: 0 !important; }`？→ 当前不涉及嵌套 property，推迟
- [Q2] mixin 展开路径中是否有循环引用风险？→ 依赖安全阀保护
