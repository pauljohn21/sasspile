## 1. @forward 前缀分隔符分析

- [x] 1.1 分析 `scan_ident` 行为：`-` 已包含在 prefix 中（如 `as d-*` → prefix = `"d-"`）
- [x] 1.2 验证 `fmt_key = format!("{p}{k}")` 正确：`"d-" + "c"` = `"d-c"` ✓
- [x] 1.3 确认无需修改代码，前缀分隔符已正确工作

## 2. CSS @import 从 @use'd 模块提升

- [x] 2.1 在 `ModuleExports` (`src/eval/env.rs`) 中新增 `css_imports: Vec<String>` 字段
- [x] 2.2 在 `load_module` (`src/eval/module.rs`) 中收集被加载模块 AST 中的 CSS @import URL
- [x] 2.3 在 `eval_use` 中将 `exports.css_imports` 追加到环境累积列表 (新增 `env.css_imports` 字段)
- [x] 2.4 在 `evaluate` / `evaluate_with_env` 中，将累积的 CSS @import 输出为 `CssNode::AtRule` 并提升到顶部
- [x] 2.5 运行 sass-spec 验证 `css-import-hoisting-from-use` 相关 case 通过
- [x] 2.6 验证核心测试无回退

## 3. @import 转发优先级

- [x] 3.1 在 `load_import` (`src/eval/module.rs`) 完成后，检测被导入模块的 `forwarded_vars` 并覆盖 local 定义
- [x] 3.2 处理 `forwarded_functions` 和 `forwarded_mixins` 的优先级
- [x] 3.3 运行 sass-spec 验证 `import-forward-precedence` 相关 case 通过
- [x] 3.4 验证核心测试无回退

## 4. @extend 跨模块增强

- [x] 4.1 菱形依赖选择器简化 — compound dedup + superselector elimination (`selector_simplify.rs`)
- [x] 4.2 增强 `:is()`/`:matches()`/`:where()` 伪选择器上下文中的 @extend（pseudo-arg 扩展）
- [x] 4.3 实现 extend scope 规则（sibling 隔离、private selector 处理）
- [x] 4.4 运行 sass-spec 验证 `extend-across-modules` 相关 case 通过
- [x] 4.5 验证核心测试无回退

## 5. 嵌套 @import 上下文修复

- [x] 5.1 在 `load_import` (`src/eval/module.rs`) 中重置选择器上下文 (`clear_selector`)
- [x] 5.2 运行 sass-spec 验证 `nested-import-context` 相关 case 通过
- [x] 5.3 验证核心测试无回退

## 6. 加载解析边界修复

- [x] 6.1 检查 `file_resolver.rs` 中 `.sass` 扩展名处理（候选顺序已正确：scss > sass > css）
- [x] 6.2 检查 `index/sass` 索引文件支持
- [x] 6.3-6.5 Placeholder extend 单文件作用域修复 (`check_extend_targets` 增加 CSS 内 placeholder 扫描)

## 7. 全量验证与归档

- [x] 7.1 运行 `SPEC_STORE_CMD=run` — snapshot 38: 7699/12131 passed
- [x] 7.2 运行 `SPEC_STORE_CMD=stats` 确认各目录通过率
- [x] 7.3 核心测试 47/48 通过（1 个 pre-existing: `test_compile_extend_placeholder`）
- [x] 7.4 更新 CHANGELOG.md (0.9.12)
- [x] 7.5 归档至 `archive/2026-09-11-selector-simplify/`

## 遗留问题

- `test_compile_extend_placeholder`：跨规则 placeholder extend 需架构改造（另行处理）
