## Why

多次 AI 迭代后，代码库积累了大量违反项目函数式 Rust 规范的代码：6 个源文件超过 500 行上限（最大 796 行）、~100 处 `for+push` 命令式累积（应使用 iterator chain）、358 处 `clone()`（其中 `exit_scope` 单函数 6 次 HashMap 深拷贝）、以及 160 个 clippy pedantic 警告。这些违规导致代码可维护性下降、性能损耗、且偏离 sasspile 的函数式设计哲学。

## What Changes

- **文件拆分**：将 6 个超 500 行的源文件拆分为 ≤ 300 行的子模块
  - `src/eval/mod.rs` (796 行) → 拆出 `exit_scope` / `hoist_css_imports` / `eval_nodes` 到独立文件
  - `src/parse/ast/display.rs` (695 行) → 按类型拆分 Display 实现
  - `src/eval/builtin/color_adjust.rs` (663 行) → 拆出 adjust/change/scale 逻辑
  - `src/eval/module.rs` (657 行) → 拆出 `load_module` / `bind_exports` / `eval_use`
  - `src/eval/builtin/color.rs` (572 行) → 拆出颜色创建函数与操作函数
  - `src/css/mod.rs` (514 行) → 拆出 `flatten_nodes` / `serialize_expanded`

- **for+push → iterator chain 重构**：将 ~100 处命令式 `for` 循环 + 可变 Vec 累积重构为 `map`/`filter`/`fold`/`try_fold`/`partition` 迭代器链
  - `builtin.rs` merge_args 系列（7 处）→ `enumerate().map().collect()`
  - `eval/mod.rs` exit_scope 传播循环（7 处）→ `into_iter().filter().for_each()`
  - `mixin.rs`（10 处）、`module.rs`（9 处）、`extend.rs`（5 处）等

- **clone() 消减**：将 358 处 clone 重构为 move 语义
  - `eval/rule.rs` eval_rule 的 6 次 scope clone → `Env::enter_rule_scope(self) -> (Env, ScopeSnapshot)` move 模式
  - `meta_ops.rs` 24 处 Value clone → move + 引用传递
  - `module.rs` 24 处 ModuleExports clone → move 语义

- **Clippy 修复**：消除 160 个 pedantic 警告
  - 15 处 `format!` appended to String → `write!`
  - 13 处不必要的 Result 包装 → 去掉 Result
  - 13 处缺少 `# Errors` 文档 → 补充
  - 13 处 items after statements → 移到前面
  - 12 处 collapsible if → `if let`
  - 3 处 if-else 链 → `match`

- **tests/ 清理**：移除 `tests/common/mod.rs` 中的 `#[cfg(test)]` 内联测试模块

## Capabilities

### New Capabilities
- `functional-compliance`: 函数式 Rust 规范执行——禁止 for+push、clone 满天飞、&mut 参数，强制 iterator chain + move 语义
- `file-size-limits`: 源文件行数限制规范——单文件 ≤ 500 行，组件 ≤ 300 行，超出必须拆分
- `clone-elimination`: clone() 消减策略——Env scope 快照从 clone 改为 move、Value 传递从 clone 改为 move/borrow

### Modified Capabilities
- `fp-architecture`: 更新函数式架构规范，增加 exit_scope 的 move 语义要求

## Impact

- **核心代码**：`src/eval/mod.rs`（Env + exit_scope 重构）、`src/eval/rule.rs`（scope 快照）、`src/eval/builtin.rs`（merge_args）、`src/eval/meta_ops.rs`（Value clone）、`src/eval/module.rs`（ModuleExports clone）、`src/eval/mixin.rs`（for+push）
- **Parser 代码**：`src/parse/ast/display.rs`（文件拆分）
- **CSS 序列化**：`src/css/mod.rs`（文件拆分 + flatten_nodes）
- **颜色系统**：`src/eval/builtin/color.rs` + `color_adjust.rs`（文件拆分）
- **测试**：`tests/common/mod.rs`（移除 #[cfg(test)]）
- **API 变更**：Env 新增 `enter_rule_scope` / `exit_rule_scope` 方法（内部 API，不影响公开接口）
- **风险**：重构涉及核心求值路径，必须逐文件验证——compile_test 43 + stage_test 10 + ep_full 121 必须保持全通过
