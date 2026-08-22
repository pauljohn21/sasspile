## Context

sasspile 的 `@directives` 子目录 sass-spec 通过率为 55%（333/605 evaluated）。272 个失败测试 + 170 个 skip 分布在 9 个子目录中。当前架构基于 `Env` + `ModuleExports` 双层结构（local/forwarded），`Evaluator::eval_use`/`eval_forward`/`eval_import` 分别处理三种模块加载指令。

核心代码位置：
- `src/eval/module.rs`：`eval_use`、`eval_forward`、`load_module`、`load_import`、`bind_exports`、`apply_config`、`resolve_file`
- `src/eval/import.rs`：`eval_import`（CSS import + SCSS import 分流）
- `src/eval/extend.rs`：`apply_extends`（选择器替换 + 占位符移除）
- `src/eval/mod.rs`：`Env` 结构体、`eval_nodes` 主循环、`eval_node` 分派
- `src/eval/rule.rs`：规则体内作用域传播

## Goals / Non-Goals

**Goals:**
- `@directives` 所有 9 个子目录 sass-spec 通过率达到 100%
- 修复 272 个失败测试（expected_error_but_ok、content_diff、missing_output、extra_output）
- 逐步解除 170 个 skip 测试
- 不破坏现有通过测试（333 个 pass + ep_full 121/121）

**Non-Goals:**
- 不修改颜色相关测试（已加入跳过列表）
- 不修改非 @directives 子目录的测试
- 不重构 `Env`/`ModuleExports` 核心数据结构（已有 local/forwarded 双层结构）
- 不引入新 crate 依赖

## Decisions

### D1: expected_error 检测——在 eval_use/eval_forward 中增加前置验证

**决策**：在 `eval_use` 和 `eval_forward` 中，在模块加载前增加配置参数验证逻辑。

**理由**：当前 `eval_use` 直接把 `with()` 配置注入环境，不检查变量是否存在于目标模块。修改方式是在 `load_module` 返回 `ModuleExports` 后、注入配置前，检查每个配置变量名是否存在于模块的 `local_vars` 或 `forwarded_vars` 中。如果不存在则报错。

**替代方案**：在 parser 层验证——但 parser 无法知道模块内容，必须在求值阶段检查。

### D2: 冲突检测——在 bind_exports 中增加同名冲突报错

**决策**：在 `bind_exports` 的 Forward 模式中，检查 forwarded_vars/forwarded_mixins/forwarded_functions 是否已存在同名成员且来源路径不同，若是则报冲突错误。

**理由**：当前 `bind_exports` 在 Forward 模式下使用"后写覆盖先写"策略，不检测冲突。sass-spec 要求 `@forward` 同名成员必须报错（除非 same_value 且来源相同）。

### D3: @extend 跨文件——增强 apply_extends 的选择器匹配

**决策**：改进 `apply_extends` 中的选择器匹配逻辑，支持更复杂的选择器模式（复合选择器、pseudo 嵌套、bogus 选择器输出）。

**理由**：当前 `apply_extends` 使用简单的 `selector.contains(target)` 匹配，不支持复合选择器拆分和 pseudo 嵌套场景。

### D4: use+import 交互——修复 CSS 输出顺序和注释处理

**决策**：在 `eval_import` 中修复 use+import 组合场景的 CSS 输出顺序，确保注释和 CSS import 的位置正确。

**理由**：当前 `load_import` 把 forwarded 合并到 local，但不处理 use 和 import 混合使用时的 CSS 输出顺序。

### D5: 特殊函数名序列化——在序列化器中增加特殊处理

**决策**：在 CSS 序列化路径中增加对 calc/clamp/expression/url/element/type 等特殊函数名的序列化处理。

**理由**：当前序列化器对特殊函数名使用通用逻辑，不区分大小写和前缀。

### D6: 加载优先级——调整 resolve_file 的候选文件顺序

**决策**：在 `try_resolve_dir` 中调整候选文件数组的顺序，确保 sass/css/partial/index 之间的查找优先级匹配 sass-spec。

**理由**：当前候选顺序为 `_{name}.scss` → `{name}.scss` → ... → `_index.scss` → `index.scss`，但 sass-spec 对某些场景有不同优先级要求。

### D7: skip 解除——逐个解除并验证

**决策**：逐个解除 skip 测试，每个解除后立即运行验证，确保不引入回归。

**理由**：批量解除 skip 风险高，逐个解除可以精确定位问题。

### D8: CSS @import 提升策略——后处理方案

**决策**：在 `evaluate()` 末尾（`apply_extends` 之后），递归扫描最终 CSS 树，提取所有 `@import` AtRule 节点到输出顶部，保持相对顺序。

**理由**：Sass 规范要求 CSS `@import`（`@import "file.css"`）出现在输出顶部，且保持源码中的相对顺序。当前 `eval_import` 把 CSS @import 作为 `CssNode::AtRule { name: "import", has_body: false }` 内联到当前位置，不提升到顶部。当 `@use` 和 `@import` 混合使用时，嵌套模块中的 CSS @import 也需要被提升到顶层。

**方案**：
```
evaluate():
  1. eval_nodes → 生成扁平 CSS
  2. apply_extends → 处理 @extend
  3. hoist_css_imports → 递归扫描，提取 @import AtRule 到顶部  ← 新增
  4. 返回 CSS

hoist_css_imports(css: &mut Vec<CssNode>):
  - 遍历 css，收集所有 AtRule { name: "import", has_body: false }
  - 从原位置移除
  - prepend 到 css 前面
  - 递归处理 AtRule/AtRoot 子节点中的 @import
```

**替代方案**：在 `eval_nodes` 中收集 @import 节点——需要改 `eval_nodes` 签名，破坏现有链式调用。

**影响范围**：`src/eval/mod.rs` 的 `evaluate()` 和 `evaluate_with_env()`。

### D9: import 嵌套策略——eval_rule 后处理方案

**决策**：扩展 `eval_rule` 的 CSS 后处理阶段，使其能处理 `AtRule` 内嵌的 `Rule` 子节点，将父选择器传播到嵌套规则中。

**理由**：当 `@import "other"` 出现在样式规则内（如 `a {@import "other"}`），被导入文件的 CSS 应该嵌套在父选择器 `a` 下。当前 `eval_rule` 的 CSS 后处理只处理 `CssNode::Rule` 子节点的选择器组合，不处理 `AtRule` 内嵌的 `Rule`。

**方案**：
```
eval_rule(selector, body, env):
  1. eval_nodes(body) → 生成 CSS
  2. 后处理：
     - CssNode::Rule → combine_selectors(selector, child_sel)  ← 已有
     - CssNode::AtRule { children } → 递归处理 children 内的 Rule  ← 新增
       - 如果 AtRule 是 keyframes：不传播父选择器
       - 如果 AtRule 是其他：传播父选择器到内部 Rule
     - CssNode::Comment → 保留  ← 新增（修复 2.4）
```

**特殊情况**：
| AtRule 类型 | 父选择器传播 | 示例 |
|------------|-------------|------|
| `@keyframes` | 不传播 | `@keyframes b { 0% {c: d} }` → 原样输出 |
| `@media` 等 | 传播 | `@media { a c {d: e} }` |
| 无 body 的 `@b c;` | 包裹 | `a { @b c; }` |

**替代方案**：在 `load_import` 中预设 `current_selector`——破坏了 `load_import` 的通用性，且无法处理 `@keyframes` 等特殊 AtRule。

**影响范围**：`src/eval/rule.rs` 的 CSS 后处理逻辑。

## Risks / Trade-offs

- [expected_error 检测可能误报] → 必须用 tracing 追踪每个错误场景的 input/output，确保只在 spec 期望错误时报错
- [bind_exports 冲突检测可能破坏现有 forward 测试] → 需在修改后运行 ep_full 121 个测试验证
- [apply_extends 改动可能影响 extend 子目录已通过测试] → 需在修改后运行 extend 全部测试
- [resolve_file 顺序调整可能影响 import/use/forward 所有测试] → 需全量运行 sass_spec_full 验证
- [170 个 skip 解除后发现大量新失败] → 按子目录分批解除，每批不超过 10 个
- [单文件 ≤ 500 行限制] → module.rs 当前接近 500 行，新增验证逻辑可能需要拆分到 module_validation.rs
- [CSS @import 提升可能破坏现有 @import 测试] → 必须在 hoist 后运行 compile_test + ep_full 验证
- [eval_rule 扩展可能影响嵌套规则输出] → AtRule 嵌套选择器组合逻辑复杂，需逐场景验证（keyframes 不传播 / @media 传播 / 无 body 包裹）
- [lexer scan_at 转义修改可能影响所有 @规则解析] → 需全量测试验证
