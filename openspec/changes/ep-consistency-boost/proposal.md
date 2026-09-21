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
| **当前** | **61/121 (50.4%)** | **color.mix RgbPercent + #{&} 插值展开** |

## 当前状态

```
总计: 121 文件
IDENTICAL: 61 (50.4%)
DIFF: 60

分类统计（基于 ep_classify_test 全量分析）:
- CAT1 @at-root/BEM & 展开: 13 文件 (carousel, cascader, collapse-item, date-picker-panel, dropdown, index, popper, select-v2, select, step, time-picker, time-select, tour)
- CAT2 CSS 变量颜色格式: 0 文件 ✅ 已修复
- CAT3 伪元素空格: 0 文件 ✅ 不存在此问题（分类误判已澄清）
- CAT4 EmptySelector/编译失败: 2 文件 (dialog, drawer)
- CAT5 选择器结构/排序差异: 32 文件
```

## 已修复问题

### ✅ color.mix 输出格式（影响 base.scss, var.scss 等）

**问题**：dart-sass 对 `color.mix()` 结果输出 `rgb(r%,g%,b%)` 格式，sasspile 输出 `#hex` 格式。

**修复**：`builtin_mix_modern` 强制使用 `ColorOutput::RgbPercent`。

### ✅ `#{& + '-x'}` 插值展开（影响 overlay.scss 等）

**问题**：`#{& + '-root'}` 中的 `&` 被当作字面字符串，未展开为父选择器。

**修复**：
1. 新增 `expand_amp_in_interp` 函数，只展开 `#{...}` 内部的 `&`
2. `eval_selector_str` 函数协调展开逻辑
3. `eval_rule` 改用 `eval_selector_str`（替代 `eval_interp_str`）

### ✅ @at-root RuleBuilder 排序（影响 35+ 文件）

**问题**：`@at-root` 提升的节点输出位置与 dart-sass 不一致。

**修复**：`RuleBuilder::build` 将 `root_nodes` 插入在父声明块之后、嵌套子规则之前。

### ✅ SCSS 嵌套函数求值（影响 display.scss 等）

**问题**：`string.unquote(map.get(...))` 未求值，输出字面表达式。

**修复**：`eval_at_rule` 参数求值前移 + calc() 内部 Sass 函数通用求值。

### ✅ var() 回退值求值

**问题**：`var(--x, map.get($map, a))` 回退值中的 Sass 表达式未求值。

**修复**：`parse_args_prefix()` + `parse_args_inner(bool)` 支持结构化参数解析。

## 剩余问题分类

### 🐛 Category 1: 选择器结构/排序差异（CAT5，32 文件）

**典型表现**：
- `backtop.scss`: `__icon` 与 `:hover` 规则顺序不同
- `image.scss`: `@extend %placeholder` 选择器分组未合并
- `button.scss`: 整体结构差异较大（17% 相似度）
- `input-number.scss`, `switch.scss`: 属性缺失或顺序差异

**子分类**：
- **1a. mixin 展开顺序**：`@include e(icon)` 生成的规则位置不对
- **1b. @extend 分组**：`@extend %placeholder` 未将多个目标合并为一个规则
- **1c. 属性序差异**：同一规则内声明顺序不同

### 🐛 Category 2: @at-root/BEM & 展开（CAT1，13 文件）

**典型表现**：
- `popper.scss`: `EmptySelector` — mixin 中 `&` 展开失败
- `cascader.scss`, `select.scss`: 嵌套规则顺序差异

**根因**：`sasspile` 对 mixin 中 `&` 在 `@at-root` 上下文中的处理与 dart-sass 不一致。

### 🐛 Category 3: EmptySelector/编译失败（CAT4，2 文件）

- `dialog.scss`: `@keyframes` 内 `calc()` 导致空块
- `drawer.scss`: 同上

## 下阶段目标

- **短期**：EP 一致性 ≥ 80/121 (66%)
- **中期**：EP 一致性 ≥ 100/121 (83%)
- **终态**：EP 一致性 121/121 (100%)

## 优先级排序（更新）

1. **P0**: CAT5 高相似度文件（backtop 85%, segmented 83%, tree 69%）— 可能只需小改动
2. **P1**: CAT5 @extend 分组（image.scss）— 需要 selector-ast 层面重构
3. **P2**: CAT1 @at-root/BEM 问题（13 文件）— 需要深入 mixin 展开逻辑
4. **P3**: CAT4 EmptySelector（2 文件）— keyframes calc 特殊处理

## 关键约束

- AGENTS.md: 禁止参照 dart-sass 实现，基于 Rust 所有权模型 + sass-spec 规范
- 单文件 ≤ 500 行
- 禁止 `unwrap()` / `clone()` 满天飞
- 每次修改需先建回归测试
- sass-spec 通过率不退化（当前基线 ~65.7%）
