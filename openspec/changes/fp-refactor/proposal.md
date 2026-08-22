## Why

sasspile 设计之初为函数式链式调用 + move 语义（零 clone），经多轮 AI 迭代后内部已严重退化为命令式风格：`Env` 被直接字段赋值、`env.clone()` 泛滥、`eval_node` 巨型 match 内联逻辑、文件按行数而非功能拆分。同时 `sasspile-macros` proc-macro crate 仅为生成三张字符串映射表而存在，增加了编译依赖和调试难度。需要一次性将架构回归函数式风格，以功能为标准重新组织代码结构，并消除不必要的宏依赖。

## What Changes

### 架构风格回归函数式

- **消除 `Env` 直接字段赋值**：所有字段修改通过 builder 方法完成（补齐 `with_depth`/`with_plain_css`/`with_loaded_modules` 等缺失方法），消除跨文件直接操作 `env.field = xxx`
- **消除 `eval_rule` 的 save/restore 模式**：用 `Env::enter_scope()` + `exit_scope()` 替代手动 clone 6 个 HashMap + 恢复
- **消除非必要的 `env.clone()`**：`bind_params`/`call_user_function`/`call_module_function` 改为 move 语义，仅在 `@content` 上下文快照保留 clone
- **后处理改为纯函数**：`apply_extends` 和 `hoist_css_imports` 从 `&mut [CssNode]` 改为 `Vec<CssNode> -> Vec<CssNode>`
- **`eval_node` match arm 提取独立函数**：`Decl`/`Comment`/`MixinDef`/`Content`/`FuncDef`/`Return`/`Extend`/`Warn`/`Error` 每个变体一个独立函数

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

## Capabilities

### New Capabilities

- `fp-architecture`: 函数式架构规范——Env 不可变 + builder 方法、eval_node 纯函数分发、后处理返回值而非 &mut、文件按功能组织
- `builtin-registry`: 内建函数注册——每个 builtin 模块自带名称映射和分派函数，消除 proc-macro 依赖

### Modified Capabilities

（无 spec 级行为变更，全部为内部架构重构，外部 API 不变）

## Impact

- **代码**：`src/eval/` 全部文件重组，`src/css/mod.rs` 拆分，`src/parse/nodes.rs` 拆分
- **依赖**：删除 `sasspile-macros` crate，移除 `syn`/`quote`/`proc-macro2` 依赖
- **Workspace**：`Cargo.toml` 移除 `[workspace]` 的 `sasspile-macros` 成员
- **测试**：所有现有测试（202/202 + sass-spec 2828）必须保持通过
- **外部 API**：`compile()`/`compile_expanded()`/`compile_file()` 等公开 API 不变
