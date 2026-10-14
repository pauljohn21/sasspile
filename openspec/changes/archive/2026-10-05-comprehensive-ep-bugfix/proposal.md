# Proposal

## Why

EP (Element Plus) 一致性仍有 27/121 文件存在 `ctx=sasspile-bug`，离 100% 一致性有显著距离。这些 bug 涵盖 6 个可分类的根因。上一轮 BEM @at-root 修复已建立完整的 trace 修复模式，当前是系统性消灭剩余 bug 的最佳时机。

## What Changes

- **A. BEM 选择器上下文修复**：anchor、color-picker-panel、descriptions、form-item 的嵌套 e/m 展开保留中间层级（+4）
- **B. var() null fallback 修复**：消除 `var(--x,null)` 泄漏，输出 `var(--x, )` 或正确 fallback（+3）
- **C. appearance vendor prefix**：添加 `-moz-appearance: none` 前缀补丁（+3~4）
- **D. :not() pseudo class 规范化**：`:not(.a,.b)` → `:not(:is(.a,.b))`（+1）
- **E. button 变量值逻辑**：修复错误变量引用导致的 disabled 色值泄漏（+1）
- **F. CSS 属性差异统一**：逐文件修复 output property 差异（+15）

预期将 sasspile-bug 从 27 降至 0。

### **BREAKING**

无 — 所有变更仅在序列化输出层面，不涉及 API 或语法定义变化。

## Capabilities

### New Capabilities
- `output/css-null-fallback`：var() 序列化正确处理 null fallback
- `output/appearance-vendor-prefix`：appearance 属性自动注入 -moz- 前缀
- `output/not-pseudo-is-wrapper`：`:not()` 多参数自动使用 `:is()` 包装

### Modified Capabilities
- `scss-eval/at-root-nesting`：扩展嵌套 BEM mixin 的中间层级保留
- `output/css-property-order`：统一多个组件的属性输出顺序
