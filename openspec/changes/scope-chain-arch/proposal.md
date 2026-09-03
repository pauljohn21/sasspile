## Why

当前 `Env` 使用扁平 `HashMap` 结构管理所有作用域变量。规则体进入时必须 `clone` 整个 `local_vars` / `local_mixins` / `local_functions` / `forwarded_*` 六张表作为 scope 快照，退出时通过 `exit_scope` 合并回去。这导致：

1. **大量必要的 clone**：每次进入 rule/mixin/function 作用域需 clone 6 个 HashMap，是剩余 clone 开销的主要来源
2. **作用域语义不精确**：SCSS 规范要求 flow control（`@if`/`@for`/`@each`）不创建新作用域，但当前实现通过 clone 实现隔离，与规范有潜在偏差
3. **`exit_scope` 逻辑复杂**：用 `std::mem::take` + 手动传播规则混合了作用域提取与变量传播语义，难以维护

基于 sass-spec 官方手册的作用域规则分析，需要引入显式的 Scope Chain 结构，以零 clone 方式管理嵌套作用域。

## What Changes

- 引入 `Scope` 结构体：单层作用域（变量 + mixin + function 表），通过 `Rc<Scope>` 链接父作用域
- `Env` 重构为 `ScopeStack`：顶层持有 `Rc<Scope>`（当前活跃作用域），每层通过 `parent` 指针链接
- **零 clone 作用域进出**：进入 rule/mixin/function 作用域时 push 新 `Scope`，退出时 pop — 无需 clone 任何 HashMap
- **`!global` 语义**：沿 scope 链向上查找全局层写入，不依赖 `global_writes` 快照
- **flow control 作用域**：`@if`/`@for`/`@each` 在**当前作用域**内直接修改变量，不创建新 scope — 符合 SCSS 规范
- **`@content` 快照**：`content_env` 改为 `Rc<Scope>` 快照（仅克隆一个指针，非整个 HashMap）
- **`exit_scope` 简化**：scope pop 时自动丢弃局部变量，仅传播 `!global` 写入和新增 mixin/function 定义
- `merge_forwarded_to_local` 保持 `std::mem::take` 模式

## Capabilities

### New Capabilities
- `scope-chain`: Scope Chain 作用域管理架构 — `Scope` 结构体 + `Rc<Scope>` 父链 + 零 clone 进出 + `!global` 向上传播 + flow control 不创建新作用域

### Modified Capabilities
- `fp-architecture`: 作用域进出不再使用 clone 快照，改为 scope push/pop，符合 move 语义优先原则

## Impact

- **核心文件**：`src/eval/env.rs`（重构 `Env` 结构 + 新增 `Scope`）、`src/eval/rule.rs`（作用域进出逻辑）、`src/eval/mod.rs`（mixin/function 调用作用域）
- **关联文件**：所有 `eval_xxx` 方法签名可能需调整（`Env` move 语义不变，但内部 scope 操作改变）
- **测试影响**：所有核心测试（compile_test 43 + stage_test 10 + ast_test 8 + common_test 5 + interp_test 15 + bs_spec 15 + ep_full 121 = 202 个）需全通过
- **sass-spec 影响**：预期通过率不变或微增（flow control 作用域语义修正可能修复部分 spec 失败）
- **性能影响**：消除 ~20+ 次 HashMap clone（每次 rule 进入），显著降低深层嵌套样式的编译开销
