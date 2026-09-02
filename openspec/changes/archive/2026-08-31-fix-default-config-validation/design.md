## Context

sasspile 当前在 `load_module` 中使用 `collect_default_vars(&ast.nodes)` 静态遍历 AST 顶层节点来验证 `@use with` 配置变量是否声明了 `!default`。这个验证在 `eval_nodes` 之前执行，导致通过 `@forward` 链转发的配置变量无法通过验证——`$a !default` 不在当前文件的顶层 AST 中，而在 `@forward` 转发的目标文件中。

当前代码流程（`module.rs` `load_module`）：
1. 注入 `pending_config`
2. `collect_default_vars` 检查 AST 顶层 ← **过早验证，失败**
3. `eval_nodes` 执行模块

## Goals / Non-Goals

**Goals:**
- 将 `!default` 验证移到 `eval_nodes` 之后，利用运行时消费记录
- 正确处理 `@forward` 链式传播（含 `as`/`show`/`hide`/`with` 变体）
- 保持 move 语义，不引入递归

**Non-Goals:**
- 不修改 `@forward` 的配置传播逻辑本身（`eval_forward` 的 pending_config 传递已正确）
- 不修改 `@use` 的模块加载逻辑
- 不处理颜色相关测试（已跳过）

## Decisions

### Decision 1: 运行时消费跟踪 vs 静态 AST 递归遍历

**选择：运行时消费跟踪**

在 `Env` 中新增 `consumed_config: HashSet<String>`。`eval_variable` 处理 `!default` 赋值时，如果从 `pending_config` 取到了值（即配置被消费），标记该 key。`eval_nodes` 完成后，检查 `pending_config` 中哪些 key 未被消费。

**理由**：
- 不需要递归解析文件——避免文件 IO 和循环引用处理
- 与 chain-reaction 的 fold 架构一致——`consumed_config` 是 `eval_nodes` fold 的副产品
- `eval_forward` 的 `pending_config` 传播已经正确工作——配置变量会被传递到目标模块

**替代方案（已否决）**：静态递归遍历 `@forward` 链的 AST——需要文件 IO、循环检测、前缀映射，且与 fold 架构矛盾。

### Decision 2: `consumed_config` 的传播方式

**选择：`consumed_config` 在 `@forward` 的 pending_config 传播时一并传递**

当 `eval_forward` 把 `pending_config` 传递给 `load_module` 时，`load_module` 执行完后，子模块的 `consumed_config` 需要回传到父模块的 `consumed_config`。

实现：`ModuleExports` 不需要新增字段——`consumed_config` 从 `final_env` 获取，在 `load_module` 验证后即可使用。

### Decision 3: 删除 `collect_default_vars`

**选择：完全删除**

`collect_default_vars` 只服务于过早的验证逻辑，移除验证后不再需要。`module_validation.rs` 文件可以完全删除或改为只保留消费验证函数。

### Decision 4: `@forward with ($a: val !default)` 的处理

当 `@forward "lib" with ($a: val !default)` 时，`eval_forward` 的 config_pairs 逻辑已正确处理了 `!default` 标记的配置传递。消费跟踪需要确保：如果 `@forward with` 中的配置变量名与上游 `pending_config` 匹配，则标记为已消费。

## Risks / Trade-offs

- **[风险] pending_config 在 @forward 传播中被消费后，父模块验证时误判为已消费** → 缓解：`consumed_config` 只记录当前模块层级消费的 key，`@forward` 传播的 config 在子模块的 `consumed_config` 中记录，需要回传
- **[风险] `@forward as prefix-*` 前缀映射导致变量名不匹配** → 缓解：`eval_forward` 的 `strip_prefix` 已经处理了前缀剥离，消费跟踪在剥离后的变量名上工作
- **[风险] `@use` + `@forward` 混合场景中消费记录可能遗漏** → 缓解：测试覆盖 `through_forward.hrx`、`distributed_vars.hrx`、`and_use` 等关键场景
- **[折中] `consumed_config` 新增 Env 字段增加内存** → 可接受：`HashSet<String>` 开销极小，且只在 `@use with` 场景有值
