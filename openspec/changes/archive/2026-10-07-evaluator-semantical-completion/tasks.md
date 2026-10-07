# Tasks

> **约束**: 所有涉及集合变换的操作必须使用 `into_iter().map().collect()` 或 `try_fold`，禁止 `for + Vec::push`。

## 0. 诊断先行

- [x] 0.1 运行 `cargo test --test bootstrap_test` 获取覆盖率数据（从~22%提升至~98%后进一步修复）
- [x] 0.2 定位真实失败点：`parse_at_query` 对 `Token::Dollar` 不处理导致 mixin body 中含变量的媒体查询静默丢失，`eval_function_body` 不执行中间变量赋值
- [x] 0.3 通过分层诊断测试（regression_media_query_with_variable、regression_function_body_with_intermediate_vars 等）确认根因
- [x] 0.4 记录结论：parser 解析 `@media ($var)` 时因 Dollar token 掉入 `_ => break` 导致后续所有节点丢失；eval 函数体执行只查找 Return 跳过了 VariableDecl

## 1. 嵌套 map 迭代修复

- [x] 1.1 scope chain 正确传递（通过 regression_three_level_nested_each 验证）
- [x] 1.2 map-get 在嵌套 @each 中正常工作
- [x] 1.3 eval_function_body 使用模式匹配替代 unwrap
- [x] 1.4 编写三层嵌套 map 迭代回归测试（eval_test.rs:549）

## 2-4. Bootstrap utility + breakpoint mixin

- [x] ~~2.x Spacing utility~~ — rx-scss 的函数式求值管线（`expand_nodes_to_events` + 通用 CSS 声明生成）自动处理了 property/values/important 展开，无需硬编码 utility 表
- [x] ~~3.x Display utility~~ — 同上，通过通用 `@each` + `@media` + selector interpolation 自动化
- [x] ~~4.x Breakpoint mixin~~ — 直接通过 `@mixin`/`@include`/`@content` + `resolve_query` 实现（通用机制，无需按 mixin 名分派）

## 5. 覆盖率验证与回归

- [x] 5.1 运行 bootstrap_test：在 @each 逗号 + parse_at_query + eval_function_body 修复后覆盖率从 ~22% 提升至 >98%
- [x] 5.2 运行全量测试：211/211 通过（parser_test 53 + eval_test 22 + integration_test 51 + 其他 85）
- [x] 5.3 bootstrap_dist_coverage 阈值已满足（>98%，远超 60% 目标）
- [x] 5.4 覆盖率提升：0.8% → ~98%（Bootstrap scss 编译对齐，缺失行从 5299 降至 ~100）
- [ ] 5.5 CodeGraph 同步：`codegraph sync`（延迟到 commit 后执行）

---

## 修复摘要（已在 commit cc05060 中实现）

### 关键 Bug 与修复

| Bug | 根因 | 修复 |
|-----|------|------|
| `@each $k in a, b` 只迭代 a | parser `parse_expression` 遇逗号停止 | 新增 `parse_each_list` 显式处理逗号分隔 |
| `.#{$k}-test` 丢失连字符 | `parse_selector` 不支持 `Token::Minus` | 选择器解析循环加入 Minus token |
| 用户定义函数中中间变量返回 null | eval 只查找 Return 跳过 VariableDecl | 新增 `eval_function_body` 依次执行函数体所有节点 |
| `@media ($var)` 使后续全部节点丢失 | `parse_at_query` 遇 `Token::Dollar` 掉入 `_ => break` | 扩展 parse_at_query 支持 `$variable` 引用 |
| media query 中变量未被求值 | Media 节点直接输出原始 query 字符串 | 新增 `resolve_query` 在 eval 阶段插值变量 |

### 新增函数（纯函数式 Rust）

- `parse_at_query` — 扩展 token 覆盖 + 变量引用
- `eval_function_body` — 消费函数体节点，返回 @return 值  
- `resolve_query` — 在 query 字符串中插值 `$var` 引用

### 测试覆盖

- 6 个回归测试常备运行（覆盖嵌套 @each、连字符选择器、函数中间变量、响应式 breakpoint mixin）
- 全部 211/211 测试通过
