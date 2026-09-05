## Why

sasspile 当前 sass-spec 通过率仅 52%（6209/12086），距离可用编译器差距较大。对照 sass-spec 规范，系统在 CSS at-rules（@keyframes/@font-face/@page 等）、meta 反射函数、颜色算法精度、@media/@supports 嵌套序列化、CSS custom properties 等关键功能存在系统性缺失。本次变更目标是按优先级分层补全所有功能，将通过率提升至 70%+。

## What Changes

**Tier 1 — 新增功能（parser 到 serializer 全链路）：**
- 实现 @keyframes 解析 + 序列化（含 vendor prefix 变体）
- 实现 @font-face 解析 + 序列化
- 实现 @page 规则解析 + 序列化（含 :left/:right/:first 伪类）
- 实现 @charset 声明处理
- 实现 @namespace 声明处理
- 实现 @layer 层叠层解析 + 序列化
- 实现 @container 容器查询解析

**Tier 2 — 反射函数补全：**
- 完善 `meta.feature-exists()` — 声明支持的 feature 集合
- 修复 `meta.content-exists()` — @content 上下文检测
- 修复 `meta.global-variable-exists()` / `meta.variable-exists()`
- 完善 `meta.apply()` 函数调用

**Tier 3 — 颜色算法修复：**
- 修复 `color.scale()` 算法（权重计算偏差）
- 修复 `color.change()` 边界值处理
- 修复 `color.invert()` HSL 空间逻辑
- `color.to-space()` / `color.to-gamut()` 精度提升

**Tier 4 — CSS 细节：**
- @supports 嵌套 + 函数输出格式
- @media 查询合并 + bubbling
- CSS custom properties (`--var: value`) 变量支持
- `@-moz-document`（css/moz_document）
- `@viewport` / `@custom-selector` / `@custom-media`
- `selector.replace()` 实现 + `selector.nest()` 列表边界

**Tier 5 — 模块系统边界：**
- @forward 的 `show`/`hide`/`as` 完整性
- @use `with()` configuration 边界
- @import 废弃兼容性（merge 到 @use 路径）

## Capabilities

### New Capabilities
- `css-keyframes`: @keyframes / @-webkit-keyframes 等 vendor prefix 变体解析+序列化
- `css-font-face`: @font-face 规则解析+序列化
- `css-page`: @page 规则 + 伪类 (:left/:right/:first)
- `css-charset`: @charset 声明处理
- `css-namespace`: @namespace 声明处理
- `css-layer`: @layer 层叠层（`@layer name { ... }` / `@layer name, name2;`）
- `css-container`: @container 容器查询
- `meta-reflection-fix`: meta.feature-exists/content-exists/global-variable-exists/variable-exists/apply 修复
- `color-algorithm-fix`: color.scale/change/invert/to-space 算法精度修复
- `css-media-supports`: @media/@supports 嵌套序列化格式
- `css-custom-properties`: CSS 变量 (--var) 定义 + #{...} 插值
- `selector-operations`: selector.replace() 实现 + selector.nest() 列表边界

### Modified Capabilities
- `color-system`: color.scale/change 算法行为调整以符合规范
- `module-system`: @forward show/hide 边界行为补全

## Impact

- **受影响的源码目录**: `src/parse/`, `src/eval/`, `src/css/`, `src/eval/builtin/`
- **新增文件预估**: 约 8-10 个模块文件（按单文件 ≤ 500 行拆分）
- **测试影响**: 预期 +1500~2500 个 sass-spec 用例通过（52% → 70%+）
- **公开 API**: 不改变公开 API，仅扩展内部功能覆盖
- **向后兼容**: 无 breaking changes，全部为功能补全
