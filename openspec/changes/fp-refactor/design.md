## Context

sasspile 设计为纯 Rust 函数式 SCSS 编译器，入口 `lib.rs` 链式调用保持完好，`stage/` 类型状态机完好。但 `eval/` 内部经多轮 AI 迭代后严重退化：

1. `Env` 被 14 处直接字段赋值（跨 4 个文件）
2. `env.clone()` 227+ 次（违反 move 语义设计）
3. `eval_node` 22 个 match arm，9 个内联逻辑不提取独立函数
4. 15 个 `impl Evaluator` 散布在 15 个文件中，`Evaluator` 是零字段空壳
5. `eval_rule` 用 30 行手动 save/restore 6 个 HashMap 管理作用域
6. 文件按行数而非功能拆分——`mixin.rs` 混了函数调用、at-rule、通用辅助
7. `sasspile-macros` proc-macro crate 仅为生成三张字符串映射表，增加编译依赖
8. 后处理 `apply_extends`/`hoist_css_imports` 用 `&mut` 就地修改

测试基线：202/202 + sass-spec 2828/5362 = 53%，重构必须保持基线不回归。

## Goals / Non-Goals

**Goals:**

- 所有 `Env` 字段修改通过 builder 方法，消除直接赋值
- `eval_node` match arm 全部委托独立函数，零内联逻辑
- 后处理改为纯函数 `Vec<CssNode> -> Vec<CssNode>`
- 文件按功能域组织，不为行数硬切
- 删除 `sasspile-macros` crate，内建函数注册内联到各模块
- `env.clone()` 仅保留 `@content` 上下文快照

**Non-Goals:**

- 不改 AST 定义（`Node`/`Value` 枚举结构不变）
- 不改公开 API（`compile()`/`compile_file()` 签名不变）
- 不改 `stage/` 类型状态机
- 不改 `lex/` 或 `parse/` 的解析逻辑
- 不提升 sass-spec 通过率（保持 2828 基线）
- 不引入新的外部依赖

## Decisions

### D1: Env builder 方法补齐 vs 作用域快照机制

**选择：补齐 builder 方法 + `enter_scope`/`exit_scope` 对**

补齐缺失的 builder 方法：`with_depth`、`with_plain_css`（已存在但未使用）、`with_loaded_modules`、`with_extends`、`with_module_cache`（已存在）。

`eval_rule` 的 save/restore 用 `enter_scope()` 创建子作用域 Env（共享 Rc 字段的浅拷贝），`exit_scope()` 从子作用域提取传播字段（命名空间变量、global writes、新增 mixin/function）合并回父作用域。

**替代方案考虑：** 完全不可变 Env + 每次 new —— 理论上更纯，但 HashMap 重建成本高。`enter_scope` 利用 Rc 字段共享读、仅 clone 变更部分，兼顾纯度和性能。

### D2: `eval_node` 分发——保持 match vs trait dispatch

**选择：保持 match 分发 + 每个 arm 委托独立函数**

`Node` 是枚举，Rust 中枚举的 match 是惯用模式。trait dispatch（`impl Eval for Node::Rule`）在 Rust 中不可行（枚举变体不是独立类型）。保持 match 作为纯分发器：

```rust
fn eval_node(node: &Node, env: Env) -> Result<(Vec<CssNode>, Env)> {
    match node {
        Node::Rule { selector, body } => eval_rule(selector, body, env),
        Node::Decl { property, value, important } => eval_decl(property, value, *important, env),
        Node::Comment(text, silent) => eval_comment(text, *silent, env),
        // ... 每个 arm 一行委托
    }
}
```

**替代方案考虑：** trait + wrapper 类型——需要为每个变体创建 newtype，增加复杂度，无实际收益。

### D3: 去掉 proc-macro——各模块自带注册

**选择：删除 `sasspile-macros`，每个 `builtin/` 子模块自带三个函数**

每个 builtin 模块（math/string/map/list/color/selector/meta）实现：
- `pub(crate) fn builtin_name(name: &str) -> Option<&'static str>` — 模块限定名 → 全局名
- `pub(crate) fn is_known(name: &str) -> bool` — 检查是否为已知函数
- `pub(crate) fn dispatch(name, args, kw, env) -> Option<Result<Value>>` — 分派到 call

`builtin/dispatch.rs` 只做转发：
```rust
pub(crate) fn module_builtin_name(name: &str) -> &str {
    math::builtin_name(name)
        .or_else(|| string::builtin_name(name))
        // ...
        .unwrap_or(name)
}
```

原来的结构体字段声明（`MathBuiltins { abs: (), div: (), ... }`）直接变成 match arm。

**替代方案考虑：** 保留宏但简化——宏本身就是生成 match，手写 match 和宏展开结果一样。宏的唯一优势是单一数据源，但 builtin 模块自带注册后，名字表和使用它的代码在同一文件里，比宏的"声明在 A 文件、生成在 B 文件"更内聚。

### D4: 后处理纯函数化

**选择：`apply_extends` 和 `hoist_css_imports` 改为消费 Vec 返回新 Vec**

```rust
// 当前
fn apply_extends(nodes: &mut [CssNode], extends: &[(String, String, bool)])
// 改为
fn apply_extends(nodes: Vec<CssNode>, extends: &[(String, String, bool)]) -> Vec<CssNode>
```

`evaluate()` 入口改为链式：
```rust
let css = eval_nodes(&ast.nodes, env)?;
let css = apply_extends(css, &extends);
let css = hoist_css_imports(css);
```

### D5: 文件按功能组织——不为行数硬切

**原则：文件名 = 功能域名。一个功能域的代码可以超过 500 行，但不相关的功能不能混在同一文件。**

具体拆分见 tasks.md。不再设 500 行硬限制，改设"功能内聚度"为拆分标准。如果一个 600 行文件全是一个功能的，不拆。

### D6: 可见性统一

所有 `eval/` 内部函数统一为 `pub(crate)`。`builtin/color_conv.rs` 的 24 个颜色转换函数从 `pub fn` 改为 `pub(crate) fn`（它们只在 `eval/` 内使用）。

## Risks / Trade-offs

- [大范围重构导致测试回归] → 分阶段推进，每阶段跑 `compile_test` + `stage_test` + `ast_test` + `common_test` 确认基线，再进下一阶段
- [`enter_scope`/`exit_scope` 设计错误导致变量作用域泄漏] → 先跑 `eval_test` 的变量作用域测试，再跑完整测试套件
- [去掉宏后名字映射遗漏] → 从宏结构体字段直接转写为 match arm，对照验证，跑 `bs_spec` 确认内建函数注册完整
- [`apply_extends` 返回值版本性能下降] → Vec move 语义零拷贝，实际不会下降；如有问题用 `cargo bench` 对比
- [`parse/nodes.rs` 拆分破坏解析逻辑] → 只移动函数到新文件，不改函数体；`use super::*` 保持导入不变
- [ep_full 38 秒测试成为迭代瓶颈] → 每阶段只跑快速测试（compile_test + stage_test + ast_test + common_test），ep_full 在关键节点验证
