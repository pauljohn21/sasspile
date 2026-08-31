## Why

sasspile 设计之初为函数式链式调用 + move 语义（零 clone），经多轮 AI 迭代后内部已严重退化为命令式风格。全量审计确认：

- **318 处 match 表达式**（排除 `matches!` 宏），分布在 20+ 文件
- **344 处 clone()**（其中 `env.clone()` 仅 5 处，其余为 Value/HashMap clone）
- **22 处直接字段赋值**（跨 6 个文件：`module.rs`/`rule.rs`/`value/mod.rs`/`mod.rs`/`module_helpers.rs`/`mixin.rs`）
- **422 处 mut 变量**
- **6 个超 500 行文件**（最大 `parse/nodes.rs` 699 行）
- **`sasspile-macros` proc-macro crate**（258 行）仅为生成三张字符串映射表，增加编译依赖
- **后处理** `apply_extends`/`hoist_css_imports` 用 `&mut` 就地修改
- **`eval_rule`** 用 30 行手动 save/restore 6 个 HashMap 管理作用域
- **`Evaluator` 是零字段空壳**，15 个 `impl Evaluator` 散布在 15 个文件

需要一次性将架构回归函数式风格，以功能为标准重新组织代码结构，并消除不必要的宏依赖。

## What Changes

### 架构风格回归函数式

- **消除 `Env` 直接字段赋值**：22 处全部改用 builder 方法（补齐 `with_depth`/`with_plain_css`/`with_loaded_modules`/`with_extends`/`with_namespaces`/`with_pending_config`/`with_global_write` 等）
- **消除 `eval_rule` 的 save/restore 模式**：用 `Env::enter_scope()` + `exit_scope()` 替代手动 clone 6 个 HashMap + 恢复
- **消除非必要的 `env.clone()`**：`bind_params`/`call_user_function`/`call_module_function` 改为 move 语义，仅在 `@content` 上下文快照保留 clone
- **后处理改为纯函数**：`apply_extends` 和 `hoist_css_imports` 从 `&mut [CssNode]` 改为 `Vec<CssNode> -> Vec<CssNode>`
- **`eval_node` match arm 提取独立函数**：`Decl`/`Comment`/`MixinDef`/`Content`/`FuncDef`/`Return`/`Extend`/`Warn`/`Error` 每个变体一个独立函数
- **`Evaluator` 空壳消除**：所有 `impl Evaluator` 方法改为自由函数，`Evaluator` 结构体删除

### 以功能为标准重新组织文件

- **`mixin.rs` 拆分**：`call_function`/`call_user_function` 移入 `function.rs`；`eval_at_root`/`eval_at_rule` 移入 `at_rule.rs`；`is_truthy` 移入 `value.rs`
- **`color.rs` 拆分**：命名颜色表移入 `colors/named_colors.rs`（纯数据）；颜色操作函数移入 `colors/color_ops.rs`
- **`builtin.rs` 拆分**：`map_param_names`/`merge_map_args` 移入 `builtin/map.rs`；calc 解析函数移入 `builtin/calc_helpers.rs`；meta 手工 match arm 移入 `builtin/meta.rs`
- **`control_flow.rs`**：移走无关的 `unit_conversion_factor` 到 `value_ops.rs`
- **`css/mod.rs` 拆分**：变换逻辑（flatten/merge/hoist）移入 `css/transform.rs`；序列化逻辑移入 `css/serialize.rs`
- **`parse/nodes.rs` 按功能拆分**：拆为 `selector.rs`/`decl.rs`/`variable.rs`/`body.rs`/`helpers.rs`
- **可见性统一**：所有内部函数统一为 `pub(crate)`，消除 `pub fn` 泄漏

### 去掉 `sasspile-macros` proc-macro crate

- **BREAKING**：删除 `sasspile-macros` workspace 成员 crate
- 每个 `builtin/` 子模块自带 `builtin_name()` + `is_known()` + `dispatch()` 函数
- `builtin/dispatch.rs` 只做转发，调用各模块的注册函数
- 消除 `syn`/`quote`/`proc-macro2` 依赖
- 消除 7 个空结构体（`MathBuiltins` 等）和 `#[derive(BuiltinRegistry)]`

### Value clone 消除

- **审计发现 339 处非 env clone() 中，大部分是 Value clone**——这是值传递的必要拷贝
- 重点消除**冗余 clone**：同一值在同一函数内 clone 多次、可 move 的值被 clone
- 不追求零 clone（Sass 语义要求值可被多次引用），但消除明显冗余

## Capabilities

### New Capabilities

- `fp-architecture`: 函数式架构规范——Env 不可变 + builder 方法、eval_node 纯函数分发、后处理返回值而非 &mut、文件按功能组织、Evaluator 空壳消除
- `builtin-registry`: 内建函数注册——每个 builtin 模块自带名称映射和分派函数，消除 proc-macro 依赖

### Modified Capabilities

（无 spec 级行为变更，全部为内部架构重构，外部 API 不变）

## Impact

- **代码**：`src/eval/` 全部文件重组，`src/css/mod.rs` 拆分，`src/parse/nodes.rs` 拆分
- **依赖**：删除 `sasspile-macros` crate，移除 `syn`/`quote`/`proc-macro2` 依赖
- **Workspace**：`Cargo.toml` 移除 `[workspace]` 的 `sasspile-macros` 成员
- **测试**：所有现有测试（202/202 + sass-spec 2828 + ep_full 121）必须保持通过
- **外部 API**：`compile()`/`compile_expanded()`/`compile_file()` 等公开 API 不变
