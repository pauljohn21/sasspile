## Why

EP (Element Plus) SCSS 编译一致性当前为 84/121 (69.4%)。诊断发现 30 个文件存在真正的 sasspile semantic bug，但当前测试管线（`ep_normalized_test`）与 EP 官方 dist（经 dart-sass + lightningcss + autoprefixer 管线生成）直接比较，混淆了 sasspile bug 与 EP 管线产物。

本 change 目标：**修复所有真正的 sasspile bug**，使 EP 语义一致性达到 ~111/121 (91.7%)。剩余 10 个文件（7 autoprefixer + 3 EP 技术改版）无法通过修 sasspile 解决。

参照系：**sass-spec 规范 + CSS 标准语义**（禁止参照 dart-sass 实现）。

## What Changes

### 核心修复

- **BEM 嵌套 `&` 作用域修复**：`&` 在 `@at-root` body 内应解析为 body selector 而非外层 scope。影响 `@mixin e()` 链式嵌套（如 `@include e(list) { @include e(item) {} }`），当前丢失中间 Block 层。
- **@keyframes 空步骤剥离**：CSS Animations spec 规定空 keyframe 步骤（`0%{}` / `100%{}`）应自动移除。sasspile 当前保留空步骤。

### 诊断能力提升

- **ep_normalized_test 管线增强**：在比较前剥离 autoprefixer 注入属性（`-webkit-user-select`、`-webkit-appearance` 等），暴露真正的语义差异而非管线产物噪声。

### 不修改（EP 管线产物，非 sasspile 职责）

- autoprefixer 注入文件 (7): checkbox-button, checkbox, input-number, radio-button, radio, slider, switch
- EP 技术改版文件 (3): option, select, select-v2（SVG mask → border 技术）

## Capabilities

### New Capabilities

- **`bem-atroot-nesting`**: 修复 `&` 选择器在 `@at-root` + `@content` 链中的嵌套作用域解析。当 `@mixin e()` 嵌套调用 `@mixin e()` 时，内层 `&` 应解析为当前 `@at-root` body 的 selector，而非外层 parent selector。基于 sass-spec parent-selector 规范。
- **`keyframes-empty-boundary-steps`**: 在序列化阶段剥离空的首尾 keyframe 步骤（`0%{}` / `100%{}`），符合 CSS Animations Level 1 规范。
- **`ep-selector-semantic-fixes`**: 修复 EP DIFF 中发现的各类 selector/value semantic bug（form-item margin-bottom 位置、input null fallback、table appearance 前缀等）。
- **`ep-pipeline-aware-diag`**: 增强 `ep_normalized_test` 测试管线，在 lightningcss normalize 后剥离 autoprefixer 注入属性，精确分类 sasspile bug vs EP 管线产物。

### Modified Capabilities

- 无（本 change 不修改已有的 sass-spec 一致性行为）

## Impact

- **受影响代码**：
  - `src/eval/rule.rs` — BEM 嵌套 `&` 作用域修复（主要修改点）
  - `src/css/serialize.rs` 或 `src/css/serialize_write.rs` — keyframes 空步骤剥离
  - `tests/ep_normalized_test.rs` — 诊断管线增强
  - `tests/ep_classify_test.rs` — 新建分类诊断测试（可选）
- **风险**：`&` 解析修改影响所有嵌套规则，需跑全量 sass-spec + 核心测试验证无回归
- **不影响的区域**：sass-spec 一致性、核心编译管线、公共 API
- **测试基线保护**：修改后必须维持 compile_test 43 + stage_test 10 + ast_test 8 + common_test 5 + bs_spec 15 = 81/81 核心测试通过
