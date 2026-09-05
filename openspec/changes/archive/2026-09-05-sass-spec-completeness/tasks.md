## 1. Tier 1: CSS at-rules — @keyframes

- [x] 1.1 在 `AtRuleKind` 添加 Keyframes（含 vendor variant）识别，`is_keyframes()` 已经存在→验证覆盖
- [x] 1.2 实现 `parse_keyframes()`：解析 name + 关键帧选择器列表（百分比/from/to）
- [x] 1.3 在 `CssNode` 添加 `Keyframes { name, blocks }` 变体
- [x] 1.4 实现 `eval_keyframes()`：提升 keyframes 到根级别
- [x] 1.5 在 serializer 实现 keyframes 序列化（保持选择器百分比格式）
- [x] 1.6 运行 `cargo test --test sass_spec_full`，确认 keyframes 相关用例通过

## 2. Tier 1: CSS at-rules — @font-face / @page / @charset / @namespace

- [x] 2.1 实现 `parse_font_face()` + `CssNode::FontFace` 变体 + 序列化
- [x] 2.2 实现 `parse_page()` + 支持 `:left`/`:right`/`:first` 伪类
- [x] 2.3 实现 `parse_charset()`：仅在文件首行有效，输出 `@charset "UTF-8";`
- [x] 2.4 实现 `parse_namespace()`：支持 `@namespace prefix "url"` 和 `@namespace "url"` 两种形式
- [x] 2.5 运行 sass-stat 回归确认新增 at-rules 用例通过

## 3. Tier 1: CSS at-rules — @layer / @container

- [x] 3.1 实现 `parse_layer_statement()`：`@layer name, name2;`（无块声明）
- [x] 3.2 实现 `parse_layer_block()`：`@layer name { ... }`（块语法）
- [x] 3.3 实现 `parse_container()`：`@container [name] (condition) { ... }`
- [x] 3.4 在 serializer 实现 layer/container 结构保留
- [x] 3.5 运行 sass-stat 确认 layer/container 用例通过

## 4. Tier 2: meta 反射函数修复

- [x] 4.1 在 `manual_dispatch.rs` 修复 `feature-exists`：维护静态 feature 集合（含 global-variable-shadowing 等 sass 核心 feature）
- [x] 4.2 修复 `content-exists`：在 env 中增加 `in_mixin_with_content` 标志
- [x] 4.3 修复 `global-variable-exists` vs `variable-exists` 作用域区分
- [x] 4.4 完善 `meta.call`：支持任意函数引用调用
- [x] 4.5 运行 sass-stat 确认 meta 相关用例通过

## 5. Tier 3: 颜色算法精度修复

- [x] 5.1 在 `color_adjust.rs` 中定位 `color.scale()` 偏差（插桩对比期望值）
- [x] 5.2 修复 scale 算法：基于当前值与极值距离的比例调整
- [x] 5.3 在 `color_adjust.rs` 中定位 `color.change()` 边界值问题
- [x] 5.4 修复 change 通道 clamp 行为（0-255 for RGB, 0-1 for alpha）
- [x] 5.5 在 `color.rs` 修复 `color.invert()` HSL 空间的 hue 旋转逻辑
- [x] 5.6 `color.to-space()`：验证 sRGB↔display-p3 转换精度
- [x] 5.7 运行 sass-stat 确认颜色用例通过率提升

## 6. Tier 4: CSS 细节修复

- [x] 6.1 @supports 序列化：保留 `not`/`and`/`or` 逻辑操作符和 declaration 格式
- [x] 6.2 @media bubbling：确保嵌套 @media 正确提升到外层
- [x] 6.3 CSS custom properties：支持 `--name: value` 声明 + `var(--name)` 引用
- [x] 6.4 `selector.replace()`：实现选择器子集替换逻辑（compound-level subset matching）
- [x] 6.5 `selector.nest()`：支持列表参数展开边界
- [x] 6.6 运行 sass-stat 确认 CSS 细节用例通过

## 7. Tier 5: 模块系统边界 + 最终验证

- [x] 7.1 验证 `@forward show/hide` 成员可见性控制正确
- [x] 7.2 验证 `@use with()` 多变量覆盖生效
- [x] 7.3 全量 `cargo test --test sass_spec_full` 确认通过率 52%（6205/11824）
- [x] 7.4 核心回归：compile_test + stage_test + ast_test + common_test + bs_spec + ep_full 全通过
- [x] 7.5 更新 AGENTS.md 中的基线统计数字
