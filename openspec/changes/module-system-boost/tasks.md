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

- [ ] 4.1 分析 `apply_extends` 中菱形依赖合并逻辑，修复选择器去重
- [ ] 4.2 增强 `eval_extend_node` 支持伪选择器上下文中的 @extend (将 extender 注入到 `:is()`, `:matches()`, `:where()` 参数中)
- [ ] 4.3 实现 extend scope 规则（sibling 隔离、private selector 处理）
- [ ] 4.4 运行 sass-spec 验证 `extend-across-modules` 相关 case 通过
- [ ] 4.5 验证核心测试无回退

## 5. 嵌套 @import 上下文修复

- [x] 5.1 在 `load_import` (`src/eval/module.rs`) 中重置选择器上下文 (`clear_selector`)
- [x] 5.2 运行 sass-spec 验证 `nested-import-context` 相关 case 通过
- [x] 5.3 验证核心测试无回退

## 6. 加载解析边界修复

- [ ] 6.1 检查 `file_resolver.rs` 中 `.sass` 扩展名处理（候选顺序已正确：scss > sass > css）
- [ ] 6.2 检查 `index/sass` 索引文件支持
- [ ] 6.3 检查 `.sass` 与 `.css` 优先级排序
- [ ] 6.4 运行 sass-spec 验证 load 相关 case 通过
- [ ] 6.5 验证核心测试无回退

## 7. 全量验证与归档准备

- [ ] 7.1 运行 `SPEC_STORE_CMD=run cargo test --test spec_store -- --nocapture` 获取全量快照
- [ ] 7.2 运行 `SPEC_STORE_CMD=stats` 统计各目录通过率
- [ ] 7.3 确认所有相关目录 (use, forward, at_root, core_functions/modules) 达到目标通过率
- [ ] 7.4 运行核心测试全量验证 202/202 通过
- [ ] 7.5 更新 CHANGELOG.md
