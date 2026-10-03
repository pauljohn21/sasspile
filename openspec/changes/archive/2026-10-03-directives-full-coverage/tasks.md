## 1. 指令注释跳过 (directive-comment-skipping) — 预估 +432 passes

- [x] 1.1 在 `parse_for` 入口添加 `skip_comments()` 调用，确保 `@for /**/ $i` 正确解析
- [x] 1.2 在 `parse_mixin` 入口添加 `skip_comments()` 调用，确保 `@mixin /**/ name()` 正确解析
- [x] 1.3 在 `parse_function` 入口添加 `skip_comments()` 调用，确保 `@function /**/ name()` 正确解析
- [x] 1.4 在 `parse_if` 入口添加 `skip_comments()` 调用，确保 `@if /**/ condition` 正确解析
- [x] 1.5 在 `parse_while` 入口添加 `skip_comments()` 调用
- [x] 1.6 在 `parse_include` 入口添加 `skip_comments()` 调用
- [x] 1.7 在 `parse_warn`/`parse_error` 入口添加 `skip_comments()` 调用
- [x] 1.8 运行核心测试确认无回归，运行 sass-spec directives/for/comment 验证修复

## 2. @function 名称大小写不敏感 (function-case-insensitive) — 预估 +168 passes

- [x] 2.1 定位 function resolver 中用户函数查找逻辑（`call_function` 或等效函数）
- [x] 2.2 在函数名比较时统一转为小写进行匹配
- [x] 2.3 确保 CSS 原生函数（element/url/expression）不受用户函数大小写变体影响
- [x] 2.4 运行 sass-spec directives/function/name/special 验证修复
- [x] 2.5 处理 vendor-prefixed CSS 函数（-a-element/-a-url/-a-expression）的 CSS 透传
- [x] 2.6 特殊处理 `-a-url()` → `url()` 的规范化（去掉 vendor 前缀）
- [x] 2.7 修复 `_moz-element()` 等下划线前缀被误判为 vendor prefix 的问题

## 3. @forward !default 变量传播 (forward-default-propagation) — 预估 +120 passes

- [ ] 3.1 定位 `evaluate_forward` 中 scope 合并逻辑
- [ ] 3.2 在合并 forwarded scope 时，检查 `!default` 标记的变量是否在 importing context 中已有定义
- [ ] 3.3 若有定义则保留 importing context 的值，否则采用 `!default` 值
- [ ] 3.4 处理 forward 链的多层传播（midstream → upstream）
- [ ] 3.5 运行 sass-spec directives/forward 验证修复

## 4. @at-root 嵌套 @use (at-root-use-nesting) — 预估 +48 passes

- [ ] 4.1 定位 at-rule 嵌套验证逻辑（`validate_at_rule_nesting` 或等效函数）
- [ ] 4.2 添加例外：`@at-root` 内部允许 `@use` 规则
- [ ] 4.3 运行 sass-spec directives/at_root/nested_import 验证修复

## 5. @use CSS 输出排序 (use-css-ordering) — 预估 +48 passes

- [ ] 5.1 定位 CSS serializer 中 at-rule 和 rule 的输出逻辑
- [ ] 5.2 确保 `@use`/`@import` 规则保持其在原始 AST 中的位置
- [ ] 5.3 确保注释与后续 CSS 规则的相对顺序不变
- [ ] 5.4 运行 sass-spec directives/use/css/order 验证修复

## 6. @use + @extend 跨模块可见性 (use-extend-visibility) — 预估 +200 passes

- [ ] 6.1 定位 `@extend` 选择器查找逻辑
- [ ] 6.2 扩展查找范围：在 `@use` 导入的模块选择器中搜索 extend 目标
- [ ] 6.3 排除 private selector（`%-name` 或 `-name`）的外部 extend
- [ ] 6.4 处理 diamond 依赖中的 extend 合并
- [ ] 6.5 处理 pseudoselector 内的 extend 链
- [ ] 6.6 运行 sass-spec directives/use/extend 验证修复

## 7. @import 变量作用域传播 (import-variable-scope) — 预估 +600 passes

- [ ] 7.1 定位 `evaluate_import` 中变量作用域的传递逻辑
- [ ] 7.2 确保 importing context 的变量在 imported file 中可见
- [ ] 7.3 确保 `!default` 变量在 importing context 有定义时被正确覆盖
- [ ] 7.4 处理嵌套 import 的作用域链（a @import b @import c）
- [ ] 7.5 处理 forward 链与 import 混合的作用域传播
- [ ] 7.6 处理同一文件 @import 两次的作用域独立性
- [ ] 7.7 运行 sass-spec directives/import/configuration 验证修复

## 8. 全量验证

- [x] 8.1 运行全部核心测试（compile_test + stage_test + ast_test + common_test + bs_spec + ep_full），确认 202/202 通过
- [ ] 8.2 运行 `SPEC_STORE_CMD=run` 全量 sass-spec，确认 directives 通过率达到 100%
- [ ] 8.3 运行 `SPEC_STORE_CMD=diff` 对比修复前后，确认无回归
- [ ] 8.4 如出现回归，定位并修复

## 当前状态 (snapshot 85)

| 子目录 | 通过率 | 失败数 |
|--------|--------|--------|
| function | 100% | 0 |
| if | 100% | 0 |
| mixin | 100% | 0 |
| other | 100% | 0 |
| while | 100% | 0 |
| for | 97.5% | 1 (CSS 嵌套) |
| forward | 97.2% | 6 (变量阴影+@import嵌套) |
| at_root | 92.6% | 2 (@import嵌套@use上下文) |
| warn | 90.9% | 1 (CSS 嵌套) |
| use | 89.9% | 27 |
| import | 68.0% | 32 |

剩余 70 个失败多为复杂架构问题（CSS 嵌套展平、@forward 变量阴影、@import 与 @use 上下文交互），需专项修复。
