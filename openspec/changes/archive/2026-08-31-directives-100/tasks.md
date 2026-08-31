## 1. expected_error_but_ok 修复（~80 个失败）

- [x] 1.1 @use with 配置验证：未定义变量检测（~15 个，`eval_use` 中增加 `config_vars` 与 `ModuleExports` 交叉验证）
- [x] 1.2 @use with 配置验证：非默认值检测（~5 个，检查变量是否带 `!default`）
- [x] 1.3 @use with 配置验证：内建模块拒绝 with（~3 个，`sass:math` 等不接受配置）
- [x] 1.4 @use with 配置验证：重复变量配置检测（~2 个，`with ($var: a, $var: b)` 报错）
- [x] 1.5 @use with 配置验证：多配置冲突检测（~4 个，`multi_configuration` 场景）
- [x] 1.6 @use with 配置验证：through_forward 配置验证（~4 个，`with/through_forward/*`）
- [x] 1.7 @use with 配置验证：无效表达式检测（~1 个，`with/invalid_expression/error`）—需表达式求值阶段检测
- [x] 1.8 @forward 冲突检测：same_value 函数/mixin（~3 个，`conflict/same_value/*`）
- [x] 1.9 @forward 冲突检测：because_of_as（~2 个，`conflict/because_of_as/*`）
- [x] 1.10 @forward 语法验证：as nothing/asterisk/no_star（~3 个，`syntax/as/*`）
- [x] 1.11 @forward 私有成员访问检测（~3 个，`inaccessible/private/*`）
- [x] 1.12 @import 冲突检测：partial/extension/all/index（~6 个，`conflict/*`）— file_resolver.rs 四种冲突检测 + cf_diag.rs .sass 过滤修复
- [x] 1.13 @import 顶层声明错误检测（~2 个，`top_level_declaration/root` + `top_level_declaration/include`）—需解析器层面检测顶层声明
- [x] 1.14 特殊函数名错误检测（~6 个，`name/error/special/*`）
- [x] 1.15 @use core_module 冲突检测（~1 个，`use/conflict/` core_module 场景）
- [ ] 1.16 @forward extend through forward 检测（~1 个，`extend/`）— 非 error 版本 3/3 通过，error 版本需要模块级 extend 作用域检查（架构改动较大，待后续）

## 2. missing_output 修复（~60 个失败）

> 修复顺序: 2.10 → 2.11/2.12 → 2.14 → 2.4 → 2.15 → 2.1-2.3/2.5 → 2.6-2.9
> 核心改动: CSS @import 提升策略（后处理）+ eval_rule 嵌套扩展 + lexer 转义处理

### 独立修复（无依赖）

- [x] 2.10 use/import escaped URL 输出缺失（~1 个，`escaped/`）— **lexer `scan_at` 增加反斜杠转义处理**，`src/lex/mod.rs` ~20 行
- [x] 2.11 use/import null 处理输出缺失（~2 个，`null/`）— 验证 `load_module` 中 `if !matches!(val, Value::Null)` 逻辑，null 配置应跳过注入使模块用 `!default` 值，可能已正确
- [x] 2.12 forward null/through_forward 输出缺失（~2 个，`null/`, `through_forward/with/null`）— 同 2.11 逻辑验证 `eval_forward` 的 config 传递
- [x] 2.14 extend bogus 选择器输出缺失（~5 个，`bogus/*`）— **`apply_extends` 检查 extender 是否以组合器（`+`/`>`/`~`）结尾，若是则跳过**，`src/eval/extend.rs` ~15 行
  - leading（`> d`）: 添加 extender ✅
  - trailing（`d +`）/only（`+`）: 丢弃 extender ✅

### 注释和声明格式修复

- [x] 2.4 use_only 注释顺序输出缺失（~4 个，`use_only/comment_order/*`）— **`eval_rule` CSS 后处理不丢弃 `CssNode::Comment` 节点**，`src/eval/rule.rs` ~10 行
- [ ] 2.15 for default 值输出缺失（~1 个，`default/`）— 嵌套属性声明 `b:` 的前缀传播到 `@for` 循环体，`src/eval/control_flow.rs` + `rule.rs` ~30 行
- [ ] 2.8 import 嵌套注释输出缺失（~1 个，`nested/with_comment`）— 同 2.4，`@import` 内联后注释应随 CSS 传播

### CSS @import 提升策略（核心改动）

- [x] 2.5 use_and_import CSS 输出缺失（~6 个，`use_and_import/*`）— **在 `evaluate()` 末尾后处理：递归扫描 CSS 树，提取所有 `@import` AtRule 到输出顶部**，`src/eval/mod.rs` ~40 行
  - 设计决策 D8: CSS @import 提升策略——后处理方案
- [x] 2.1 use+import 交互：use_into_use 输出缺失（~2 个，`scope/use_into_use_and_*`）— 依赖 2.5 的 @import 提升框架
- [x] 2.2 use+import 交互：import_into_use 输出缺失（~2 个，`scope/use_and_import_into_diamond_extend`）— 依赖 2.5
- [x] 2.3 use+import 交互：isolated_through_import（~1 个，`scope/isolated_through_import`）— 依赖 2.5

### import 嵌套修复（依赖 eval_rule 扩展）

- [x] 2.6 import 嵌套 use 输出缺失（~2 个，`import/*`）— **`eval_rule` CSS 后处理扩展：处理 `AtRule` 内嵌 `Rule` 子节点的选择器嵌套**，`src/eval/rule.rs` ~40 行
  - 设计决策 D9: import 嵌套策略——eval_rule 后处理方案
- [x] 2.7 import 嵌套 @规则输出缺失（~4 个，`nested/at_rule/*` — rule_child, declaration_child, childless, keyframes）— 依赖 2.6 的 eval_rule 扩展
  - rule_child: `a {@import "other"}` → `other` 含 `@b { c {d: e} }` → 输出 `@b { a c { d: e; } }`
  - declaration_child: `a {@import "other"}` → `other` 含 `@b {c: d}` → 输出 `@b { a { c: d; } }`
  - childless: `a {@import "other"}` → `other` 含 `@b c;` → 输出 `a { @b c; }`
  - keyframes: `a {@import "other"}` → `other` 含 `@keyframes` → 输出忽略父选择器
- [ ] 2.9 import 空白处理输出缺失（~2 个，`whitespace/modifier/args/*` — before_close_paren, after_open_paren）— CSS @import 序列化保留原始空白格式

### 其他

- [ ] 2.13 forward import_to_forward 输出缺失（~1 个，`import_to_forward/nested/function`）— `@import` 内联后 `@forward` 转发的成员对 import 可见
- [ ] 2.16 import member inaccessible（~1 个，`member/inaccessible/nested/function`）—实际为 missing_output

## 3. content_diff 修复（~80 个失败）

- [ ] 3.1 @extend 跨文件选择器传递：extended/extended（~2 个，`extended/extended/*`）
- [ ] 3.2 @extend 钻石依赖：diamond/merge + diamond/dependency（~2 个）
- [ ] 3.3 @extend optional_and_mandatory（~3 个）
- [ ] 3.4 @extend scope 隔离（~3 个，`scope/sibling`, `scope/downstream`, `scope/diamond`）
- [ ] 3.5 @extend upstream 传递（~2 个，`upstream/*`）
- [ ] 3.6 @extend midstream_extend_within_pseudoselector（~4 个）
- [ ] 3.7 @extend pseudo/into_pseudo/extends_after（~1 个）
- [ ] 3.8 use 变量赋值差异：namespaced/default（~1 个）
- [ ] 3.9 use 全局变量赋值：global/nested/local（~1 个）
- [ ] 3.10 forward 变量遮蔽：shadowed/through_forward（~1 个）
- [ ] 3.11 forward 嵌套遮蔽：shadowed/nested/through_forward（~2 个）
- [ ] 3.12 forward 覆盖：override/override/*（~3 个）
- [ ] 3.13 forward 优先级：precedence/*（~2 个）
- [ ] 3.14 forward with/non_overridable 差异（~1 个）
- [ ] 3.15 forward with/private/different 差异（~1 个）
- [ ] 3.16 forward through_forward/as 差异（~1 个）
- [ ] 3.17 forward variable_exists 差异（~1 个）
- [ ] 3.18 forward dash_insensitive 差异（~1 个）
- [ ] 3.19 import 中间流定义：midstream_definition/*（~2 个）
- [ ] 3.20 import 重复导入：import_twice/*（~2 个 — with_change, still_changes_in_same_file）
- [ ] 3.21 import 遮蔽：separate_file/shadowing/through_forward（~3 个 — direct, through_forward, nested/local, nested/global）
- [ ] 3.22 import prefixed_as 差异（~1 个，`prefixed_as/`）
- [ ] 3.23 import 注释处理：comment/modifier/args/*（~2 个 — after_open_paren/loud, before_close_paren/loud）
- [ ] 3.24 import CSS 输出：css_import_after_style_rule（~1 个）
- [ ] 3.25 import 加载优先级：load/precedence/*（~6 个 — sass_before_css, import_only/* 5 个）
- [ ] 3.26 function 特殊函数名序列化：name/special/*（~8 个）
- [ ] 3.27 use distributed_vars 差异（~2 个，`distributed_vars/*`）
- [ ] 3.28 for in_declaration 输出格式（~1 个）
- [ ] 3.29 import same_file/nested/indirect 差异（~3 个 — same_file, nested, indirect/through_forward）
- [ ] 3.30 import unrelated_variable 差异（~1 个，`unrelated_variable/`）

## 4. skip 解除（~170 个 skip）

- [ ] 4.1 解除 directives/at_root 的 21 个 skip（分 3 批）
- [ ] 4.2 解除 directives/mixin 的 29 个 skip（分 3 批）
- [ ] 4.3 解除 directives/if 的 19 个 skip（分 2 批）
- [ ] 4.4 解除 directives/forward 的 30 个 skip（分 3 批）
- [ ] 4.5 解除 directives/use 的 30 个 skip（分 3 批）
- [ ] 4.6 解除 directives/for 的 21 个 skip（分 3 批）
- [ ] 4.7 解除 directives/extend 的 3 个 skip
- [ ] 4.8 解除 directives/function 的 10 个 skip
- [ ] 4.9 解除 directives/import 的剩余 skip（如有）
- [ ] 4.10 修复 skip 解除后发现的新失败

## 5. 验证与回归测试

- [x] 5.1 每批修复后运行 `cargo test --test compile_test`（43 个核心测试不回归）
- [x] 5.2 每批修复后运行 `cargo test --test ep_full`（121 个不回归）
- [ ] 5.3 每批修复后运行 `cargo test --test sass_spec_full`（检查 @directives 通过率提升）
- [ ] 5.4 阶段 1 完成后验证 @directives 通过率达 ~68%（expected_error 修复）
- [ ] 5.5 阶段 2 完成后验证 @directives 通过率达 ~78%（missing_output 修复）
- [ ] 5.6 阶段 3 完成后验证 @directives 通过率达 ~91%（content_diff 修复）
- [ ] 5.7 阶段 4 完成后验证 @directives 通过率达 100%（skip 解除）
- [ ] 5.8 全量 sass-spec 验证（sass_spec_full 不回归）
- [x] 5.9 AGENTS.md 通过率基线更新（2828/5362 = 53%）
- [x] 5.10 codegraph sync 同步索引

## 当前进度

| 子目录 | FAIL 数 | 主要类型 | 状态 |
|--------|---------|----------|------|
| forward | ~15 | content_diff + inaccessible | 76% 通过 |
| import | 32 | content_diff(24) + missing_output(7) + expected_error(2) | conflict 5/5 修复 |
| use | ~33 | content_diff + missing_output | 待修 |
| extend | ~30 | content_diff + missing_output | 待修 |
| 其他 | ~30 | various | 待修 |
