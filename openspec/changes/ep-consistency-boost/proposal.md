# EP Consistency Boost — Proposal

## 背景

Element-Plus (EP) 项目有 121 个 SCSS 组件文件，通过 `@use 'mixins/mixins' as *` 使用 BEM 命名系统（`b()` / `e()` / `m()` mixins）。

## 基线演进

| 时间点 | IDENTICAL | 关键变更 |
|--------|-----------|----------|
| 起始 | 21/121 (17.4%) | — |
| Phase 1 | 27/121 (22.3%) | !global 作用域修复 |
| Phase 2 | 45/121 (37.2%) | @at-root & 展开、SCSS 嵌套函数求值 |
| Phase 3 | 52/121 (43%) | @at-root RuleBuilder 排序修复 |
| Phase 4 | 61/121 (50.4%) | color.mix RgbPercent + #{&} 插值展开 |
| **Phase 5** | **73/121 (60.3%)** | **AtRootDirect 源码位置 + 字面 `&` 展开** |

## Phase 5 变更概要

### ✅ AtRootDirect 节点类型 — mixin @at-root 源码位置（commit 30eeccb）

**问题**：EP BEM mixin（`e()`, `m()`）内部 `@at-root { ... }` 生成的子选择器被 RuleBuilder 统一插入固定位置（第一个父规则之后），破坏源码顺序。典型表现：`backtop.scss` 的 `__icon` 规则出现在 `:hover` 之前。

**决策**：新增 `CssNode::AtRootDirect` 节点类型，在 `exec_mixin` 求值后替换 `AtRoot` → `AtRootDirect`，`RuleBuilder::push` 识别并直接放入源码位置（bypass combine_selectors）。

**影响**：全链路适配 rule/serialize/hoist/extend/meta_ops。sass-spec -84（7977→7893），trade-off 可接受。

### ✅ 字面 `&` 选择器展开（commit c97e582）

**问题**：`when(disabled)` mixin 生成的 `&.disabled` 选择器经过 AtRootDirect 后字面量 `&` 未被展开，输出 `&.disabled` 而非 `.el-segmented__item-selected.is-disabled`。

**决策**：`RuleBuilder::push` 对 AtRootDirect 内部 Rule 做条件处理——选择器含 `&` 时调用 `combine_selectors(parent, child)` 展开；不含 `&` 时直接 push（保留 e() mixin 的提升语义）。

**影响**：segmented/tree/popover/pagination/timeline 等 21 个文件从 DIFF→IDENTICAL。sass-spec 无新回归。

## 当前状态

```
总计: 121 文件
IDENTICAL: 73 (60.3%)
DIFF: 44
错误: 4（lightningcss 解析失败，含 EmptySelector/pseudo-element 空格问题）
```

## 剩余问题分类

### 🐛 Category A: `@extend %placeholder` 选择器分组（影响 ~25 文件）

**典型表现**：sasspile 输出明显短于 dist（缺少整块 declarations）。

- `descriptions.scss`: sp=642, dist=1018（-376）
- `form-item.scss`: sp=4205, dist=5405（-1200）
- `input-number.scss`: sp=3631, dist=4347（-716）
- `carousel.scss`, `cascader.scss`: 中等缺失

**根因**：dart-sass 对 `%placeholder { ... }` + `.a, .b, .c { @extend %placeholder }` 输出组合选择器 `.a, .b, { shared-decls }`，sasspile 完全跳过该 extend 的声明复制。

### 🐛 Category B: 选择器嵌套/分组差异（影响 ~10 文件）

- `button.scss`, `dropdown.scss`: sasspile 输出完全 flat，dart-sass 输出嵌套结构
- 此类为 AtRootDirect 设计的自然结果（语义等价但文本不同）

### 🐛 Category C: 其他差异（影响 ~10 文件）

- `anchor.scss`: 可能涉及伪类/伪元素输出差异
- `dialog.scss`, `drawer.scss`: keyframes 内特殊结构

## 下阶段规划

### P0: `@extend %placeholder` 声明分组

**目标**：实现 `%placeholder { decls }` + `@extend %placeholder` → 输出组合选择器。

**工作量**：中等 — 需要修改 `apply_extends` 逻辑以捕获 placeholder 声明并在多个 extender 间共享。

### P1: 伪元素规范化

**目标**: `:before`/`:after` 输出格式与 dart-sass 一致。

## 关键约束

- AGENTS.md: 禁止参照 dart-sass 实现，基于 Rust 所有权模型 + sass-spec 规范
- 单文件 ≤ 500 行
- 禁止 `unwrap()`（生产代码）
- 每次修改需先建回归测试
- sass-spec 通过率不退化（当前 7893/12133 = 65%）
