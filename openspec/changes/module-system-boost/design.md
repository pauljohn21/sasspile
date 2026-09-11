## Context

sasspile 模块系统当前实现覆盖了 `@use` / `@forward` / `@import` 的基本语义，但 sass-spec 中 4 个相关目录（`directives/use`、`directives/forward`、`directives/at_root`、`core_functions/modules`）仍有 63 个失败 case。主要根因分为 5 类：

1. **@forward 前缀分隔符缺失**：`fmt_key` 未插入 `-`，导致 `d-c` 存为 `dc`
2. **CSS @import 未从 @use'd 模块提升**：@use 模块内的 `@import "file.css"` 未出现在最终输出
3. **@import 转发优先级未实现**：@forward 转发成员的优先级应高于 @import 文件本地定义
4. **@extend 跨模块不完整**：菱形依赖合并、伪选择器内嵌套缺少支持
5. **嵌套 @import 上下文限制**：CSS rule 内 @import 的文件中不允许 @use

### 当前代码位置

| 组件 | 文件 | 关键函数 |
|------|------|----------|
| 模块绑定 | `src/eval/module_helpers.rs` | `bind_exports`、`fmt_key` |
| 模块加载 | `src/eval/module.rs` | `load_module`、`eval_use` |
| 导入处理 | `src/eval/import.rs` | `eval_import` |
| 转发处理 | `src/eval/forward.rs` | `eval_forward` |
| 类型定义 | `src/eval/env.rs` | `ModuleExports`、`Env` |
| 继承处理 | `src/eval/extend.rs` | `apply_extends`、`eval_extend_node` |
| 文件解析 | `src/eval/file_resolver.rs` | `resolve_file` |

## Goals / Non-Goals

**Goals:**
- 修复 63 个失败 case 中的尽可能多的 case（预估 40-50 个）
- 将所有相关目录推至 95%+ 通过率
- 保持已有 202/202 核心测试通过

**Non-Goals:**
- 不重构整个模块系统架构
- 不改变 @use / @import 的基本语义
- 不处理 libsass 不支持的 case
- 不优化性能（正确性优先）

## Decisions

### Decision 1: @forward 前缀分隔符

**问题**：`@forward "upstream" as d-*` 中 prefix = `"d"`，但 `fmt_key` 只做简单拼接 `format!("{p}{k}")`，结果 `"d" + "c"` = `"dc"` 而非 `"d-c"`。

**方案**：修改 `bind_exports` 中 `fmt_key` 闭包：
```rust
// Before
let fmt_key = |k: &str| -> String { 
    prefix.map_or_else(|| k.to_string(), |p| format!("{p}{k}")) 
};
// After
let fmt_key = |k: &str| -> String { 
    prefix.map_or_else(|| k.to_string(), |p| format!("{p}-{k}")) 
};
```

**理由**：SCSS 规范中 `as prefix-*` 的 `-` 是语法的一部分。解析器只捕获 prefix 名（不含 `-`），应用时需补回分隔符。这是最简单的修复，一行改动。

**备选**：修改解析器存储完整前缀 `"d-"`，但这需要改动 AST 和 Display trait，影响更大。

### Decision 2: CSS @import 提升

**问题**：当 `@use` 的模块内部有 CSS `@import` 时（如 `@import "midstream.css"`），这些 `@import` 需要被收集并按依赖顺序提升到最终输出的顶部。

**方案**：
1. `ModuleExports` 新增 `css_imports: Vec<String>` 字段
2. `load_module` 执行时收集被加载模块 AST 中的 CSS @import URL
3. `eval_use` 时将 `exports.css_imports` 追加到环境的累积列表中
4. 新增 `env.css_imports: Vec<String>` 字段
5. 最终在 `evaluate` / `evaluate_with_env` 中，将所有累积的 CSS @import 输出为 `CssNode::AtRule` 并提升

**理由**：CSS @import 提升是 Sass 规范要求，且 `hoist.rs` 已有类似的提升逻辑。将收集推迟到模块加载阶段是被动的做法，确保不遗漏任何层级的 @import。

**备选**：在 `hoist_css_imports` 中递归扫描 ModuleExports 的 CSS。但这需要在 hoist 阶段访问 module_cache，增加耦合。

### Decision 3: @import 转发优先级

**问题**：sass-spec 规范要求 "Forwarded definitions take precedence over local definitions through imports"。即被 @import 的文件中 @forward 转发的成员，优先级高于导入文件中本地定义的变量。

**方案**：
1. `ModuleExports` 已有 `forwarded_vars` / `local_vars` 分离
2. `load_import` 返回后，检测被导入模块是否有 `forwarded_vars`
3. 若有，在合并到调用者环境时，forwarded 成员覆盖 local 成员（而非当前 local 优先）

**理由**：这是 Sass 模块系统的核心语义 —— 转发成员代表"下游模块的定义"，应优先于当前文件的本地定义。

**风险**：可能影响现有通过 case。需全量测试验证。

### Decision 4: @extend 跨模块增强

**问题**：
- 菱形依赖中多个模块 `@extend` 同一选择器时应合并
- `:is(in-midstream) {@extend in-upstream}` 中 extender 需嵌套进伪选择器

**方案**：
1. **菱形合并**：`apply_extends` 使用 `module_selectors` 做 scope 检查。确保同一选择器被多个模块 extend 时，最终 selector list 去重合并。
2. **伪选择器内 @extend**：增强 `eval_extend_node`，当当前选择器包含伪选择器（`:is()`, `:matches()`, `:where()`）时，将 extendee 注入到伪选择器参数列表中。

**理由**：当前 `eval_extend_node` 只处理扁平选择器，遇到伪选择器上下文时无法正确注入。

**风险**：伪选择器解析复杂，需精确处理嵌套括号。

### Decision 5: 嵌套 @import 上下文

**问题**：`a { @import "other"; }` 中 `@import` 的文件包含 `@use` 时报错 "This at-rule is not allowed here"。

**方案**：`load_import` 创建新解析上下文时，重置 `in_body` 和 `saw_other_rule` 标志。

**理由**：`@import` 的文件在新的顶层上下文中解析，不受外部 CSS rule 限制。`with_plain_css` 已做类似处理。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| 前缀修复影响已有通过 case | 全量测试验证，若有回退则分析原因 |
| @import 提升改变输出顺序 | 严格按依赖顺序排列，保持已有顺序不变 |
| @import 优先级规则影响其他目录 | 限定在 `load_import` 内，不影响 `load_module` |
| 伪选择器内 @extend 复杂度 | 先处理简单 case，复杂 case 留后续迭代 |
