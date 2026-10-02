## Why

EP（Element Plus）一致性当前在 **83/121 (68.6%)**。之前完成的 Phase 1-7 包含 BEM mixin 上下文修复、@at-root/@content 顺序、@extend %placeholder、@at-root/@content 伪类括号分割修复、字面量 Value crash 修复等。sass-spec 从 7921 → 7927 (+6)。

剩余 38 DIFF 文件经诊断分两类：**sasspile bug（可修复）** 和 **EP 管线特性（非 sasspile 编译错误）**。

## What Changes

本轮已完成 + 下一步规划：

### 已完成
1. **`:not()` 括号分割修复**（`combine_selectors` bracket-aware）：`:not(.a, .b)` 括号内逗号不再破坏选择器语义
2. **字符串字面量 crash 修复**（`parse_literal_arg`）：1-char string slice 不再 panic
3. **sass-spec 净提升 +6**（7921 → 7927），零回归

### 剩余可修复（sasspile bug）
4. **`@at-root` selector 解析边界**：descriptions 系列 `e(header)` 在 `m($size)` 内 @at-root 上下文丢失 → 嵌套选择器不完整
5. **`rgba()`/`rgb()` var() fallback 展开**：input-number 系列 `rgba(#xxx, 0)` 函数参数未正确解压

### 不可在 normalize 测试中修复（EP 管线特性）
- `-webkit-user-select:none`（6 files） — autoprefixer
- `translate(0,0)` → `translate(0)`（5 files） — lightningcss 简化
- `calc(X - 1px * 2)` → `calc(X - 2px)`（3 files） — lightningcss 简化
- `--lightningcss-light`/`--lightningcss-dark` media vars（2 files） — lightningcss color-scheme

**BREAKING**：无 breaking change。所有改动保持 202/202 核心测试 + sass-spec 通过。

## Capabilities

### New Capabilities
- `selector-comma-preserve`: 含逗号的选择器列表在括号内正确分割
- `literal-arg-safety`: parse_literal_arg 1-char string 安全处理
- `bem-at-root-nesting`: BEM mixin 嵌套中 @at-root 选择器上下文保留

### Modified Capabilities
无现有 capability 的需求变更。

## Impact

- **代码文件**: `src/eval/rule.rs`（+80 行 bracket-aware split）、`src/eval/value/mod.rs`（+4 行 guard）、`src/eval/mixin.rs`（待修复 at-root）、`src/eval/builtin/rgb_rgba.rs`（待修复 var fallback）
- **测试**: tests/ep_normalized_test.rs（基线 83，目标 ~121 减去管线差异）
- **风险**: mixin/resolver 层改动需 SPEC_STORE_CMD=run 全量验证
- **工作量**: ~5 剩余 sasspile-bug file × 1-2 个独立 task
