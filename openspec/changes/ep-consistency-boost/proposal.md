# EP Consistency Boost — Proposal

## 背景

Element-Plus (EP) 项目有 121 个 SCSS 组件文件，通过 `@use 'mixins/mixins' as *` 使用 BEM 命名系统（`b()` / `e()` / `m()` mixins）。经过前期 `!global` 作用域修复后，sasspile EP 一致性从 21/121 (17.4%) 提升至 27/121 (22.3%)。

剩余 87 个 DIFF 和 7 个 lightningcss 解析错误，涉及**三类核心问题**。

## 基线

| 指标 | 值 |
|------|-----|
| 当前一致 | 27/121 (22.3%) |
| DIFF | 87 |
| lightningcss FAIL | 7 |
| sass-spec | 7971/12133 (65.7%) |

## 问题分类（基于 ep_diff_test 分析）

### 🐛 Category A: `&` 父选择器未展开（约 35-40 文件）

**根因**：`m()` BEM mixin 用 `$selector: &` 捕获父选择器，然后通过 `@at-root { #{$currentSelector} { ... } }` 输出规则。sasspile 的 `RuleBuilder::push` 对 `@at-root` 节点走 `root_nodes.extend` 路径，**未将 `&` 替换为实际父选择器**。

**典型表现**（选择器 sasspile vs expected）：
- `.el-avatar` 下 `&--circle` → `.el-avatar--circle`
- `.el-badge__content` 下 `&.is-fixed` → `.el-badge__content.is-fixed`
- `.el-button` 下 `&.is-plain` → `.el-button.is-plain`
- `.el-breadcrumb` 下 `&::before` → `.el-breadcrumb::before`
- `.el-container` 下 `&.is-vertical` → `.el-container.is-vertical`
- `.el-form-item` 下 `&--large` → `.el-form-item--large`
- `.el-loading-parent` 下 `&--relative` → `.el-loading-parent--relative`
- `.el-affix` 下 `&--fixed` → `.el-affix--fixed`
- `.el-collapse-item` 下 `&.is-disabled` → `.el-collapse-item.is-disabled`
- `.el-input-number` 下 `&.is-left` → `.el-input-number.is-left`
- `.el-descriptions` 下 `&--inline` → `.el-descriptions--inline`
- `.el-divider` 下 `&--horizontal` → `.el-divider--horizontal`
- `.el-input` 下 `&.is-focused` → `.el-input.is-focused`
- `.el-link` 下 `&.is-hover-underline` → `.el-link.is-hover-underline`
- `.el-cascader-panel` 下 `&.is-bordered` → `.el-cascader-panel.is-bordered`
- `.el-carousel` 下 `&--horizontal` → `.el-carousel--horizontal`

**影响**：这是跨文件共性问题，几乎所有使用 BEM `m()` mixin 并配合 `#{&}` 选择器扩展的组件都受影响。

### 🐛 Category B: CSS 序列化格式差异（约 20-25 文件）

**子类 B1 — 颜色名称 vs hex**：
- `white` → `#ffffff`
- `black` → `#000000`
- `rgba(0, 0, 0, 0)` → `transparent`

**子类 B2 — CSS 函数名大小写**：
- `scalex(0)` → `scaleX(0)`
- `translatex(-50%)` → `translateX(-50%)`
- `rotatez(0deg)` → `rotateZ(0deg)`

**子类 B3 — 选择器中 `&` 嵌套展开不完整**（与 Category A 独立的问题）：
- `.el-color-picker:hover:not(.is-disabled .el-color-picker__trigger, .el-color-picker__source)` → `.el-color-picker:hover:not(.is-disabled, .is-focused) .el-color-picker__trigger`
- `.el-checkbox-button__inner` 选择器上下文位置不同

**子类 B4 — `@keyframes` 中 `%` 转义**：
- `\%` → `0%`
- `\30 0\%` → (移除)

**影响**：不改变语义但字符串对比失败，需要输出规范化（dart-sass 行为对齐）。

### 🐛 Category C: SCSS 函数/插值未求值（约 10-15 文件）

**根因**：`sass:string.unquote()` / `map.get()` 等 SCSS 内置函数调用未被识别并求值，将字面表达式 `string.unquote(map.get($map, $key))` 输出到 CSS。

**典型表现**：
- `display.scss`: `@media only screen and string.unquote(map.get($map, $key))` → `@media only screen and (max-width: 767px)`
- `input.scss`: `calc(getCssVar("input-otp-size") - 4px)` → `calc(var(--el-input-otp-size) - 4px)` — 嵌套 `getCssVar` 未展开
- `select-dropdown-v2.scss`, `input-otp.scss`: `getCssVar` 嵌套调用未展开

**影响**：文件级输出完全错误（语义不等价），需要优先修复。

### ⚡ lightningcss FAIL（7 文件）

不是 sasspile 的 bug，是 lightningcss 解析器对合法 CSS 的兼容限制：
- `col.scss`: UnexpectedToken(Ident("string")) at line 609
- `date-picker-panel.scss`: EmptySelector
- `display.scss`: UnexpectedToken(Ident("string"))
- `index.scss`: UnexpectedToken(Ident("string"))
- `popper.scss`: EmptySelector
- `step.scss`: EmptySelector
- `table.scss`: PseudoElementExpectedIdent

**应对**：这些问题中部分会随 Category B/C 修复而消失（如 `string.unquote`）；EmptySelector 可能是 sasspile 输出了空选择器需要修复。

## 目标

- EP 一致性 ≥ 80/121 (66%)
- 核心测试不退化
- sass-spec 不退化

## 优先级排序

1. **P0**: Category A — `&` 父选择器展开（覆盖面广，改动集中）
2. **P1**: Category C — SCSS 嵌套函数求值（semantic correctness）
3. **P2**: Category B — CSS 格式化对齐（`white`→`#fff`, `scaleX` 等）
4. **P3**: lightningcss 兼容（部分依赖 B/C 修复）

## 关键约束

- AGENTS.md: 禁止参照 dart-sass，只能基于 sasspile 现有函数式架构 + sass-spec 规范
- 单文件 ≤ 500 行
- 禁止 `unwrap()` / `clone()` 满天飞
- 每次修改需先建回归测试
