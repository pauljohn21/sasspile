## Why

`main` 分支积累了一整套基于 `Rc<Scope>` GC 风格的 SCSS 编译器实现（sass-spec 8003/12133 = 66%），但架构对 AI 协作极度不友好 —— 状态分散在 5 个 Observer × 5 个 State 枚举，所有权规则互相矛盾，AI 回退到 GC 思维。

`rx` 分支采用 **rxrust 响应式架构**（scan_map + flat_map 算子链），状态统一在单一 `CompileState`，零 clone/零 Arc<Mutex>。当前 rx 已通过 32 个测试验证了以下能力：
- `@mixin/@include`（fold 参数替换 + 默认值回退）
- `@each/@for`（单行 inline + 多行 body 收集）
- `@if/@use/@forward` 指令消费
- 选择器嵌套展开（后代选择器 + `&` 父引用 + 深层嵌套）
- CSS 格式化输出（缩进/换行/空行清理）

**目标**：将 `main` 分支的关键能力逐项迁移到 `rx` 分支，以 rxrust 响应式风格重新实现（不是 cherry-pick 代码）。最终 rx 分支成为唯一开发主线。

## What Changes

### 架构约定（rx 铁律）

1. **单一状态载体**: `CompileState` + `scan_map` reducer，禁止 `Rc<RefCell>`
2. **三阶段管道**: from_iter → scan_map + flat_map → collect/last → subscribe
3. **禁止命令式累积**: 不用 `let mut v = Vec; for x in items { v.push(x) }`，改用 `flat_map(from_iter)`
4. **禁止 match 嵌套 if/else 链**: 用独立 helper 函数
5. **单文件 ≤ 500 行**: 超出必须拆分子模块
6. **全量 tracing span**: 疑似路径入口/出口必须 `info_span!`

### 能力迁移矩阵（10 capabilities）

| # | capability | main 实现 | rx 现状 | 策略 |
|---|---|---|---|---|
| 1 | `css-ast-construction` | CssNode 枚举（Rule/Declaration/AtRoot/AtRule） | ❌ 缺失 | 新建 CssNode + scan_map 构建 |
| 2 | `css-serializer` | `serialize_write.rs` 格式化输出 | ✅ 基础 formatter | 逐步替换（AST 驱动格式） |
| 3 | `selector-ast` | `selector_ast.rs` + `selector_parser.rs` | ✅ stack 拼接 | 引入 Selector AST |
| 4 | `selector-extend` | `selector_extend/` 5 个文件 | ❌ 缺失 | 全链路 @extend 实现 |
| 5 | `variable-resolution` | `eval/variable.rs` Scope Chain | ✅ HashMap 扁平 | 保留 rx 扁平模型 |
| 6 | `function-call-eval` | `eval/builtin/` 全局函数（color/math/list...） | ❌ 缺失 | 按 sxrust 风格 split 实现 |
| 7 | `at-root-hoisting` | `CssNode::AtRootDirect` + `push_atroot_direct` | ❌ 缺失 | 新增 AtRootDirect Phase 状态 |
| 8 | `module-loading` | `@use`/`@forward` 文件系统加载 | ✅ parse 名新增 | 接入 tokio async 加载 |
| 9 | `mixin-content-block` | `@content` 传播 | ❌ 缺失 | 收集态新增 Content 分支 |
| 10 | `source-map-tracing` | `Span` 追踪源码位置 | ❌ 缺失 | 选择性引入（rx 风格 span） |

### 新增文件规划（rx 响应式分模块）

```
src/directive/
├── pipeline.rs          # ✅ 主入口 + scan_map + 状态机（已完成, <462行 -> 拆分）
├── state.rs             # ✅ CompileState + Collecting（已完成）
├── finalize.rs          # 🆕 finalize_collecting 提取（避免 pipeline.rs 超 500 行）
├── selector_stack.rs    # 🆕 选择器嵌套栈逻辑提取
└── format.rs            # 🆕 format_css 提取

src/css/
├── mod.rs               # 🆕 CssNode 枚举（Rule/Declaration/AtRoot/AtRule）
├── serialize.rs         # 🆕 AST → String（格式化输出）
└── selector.rs          # 🆕 选择器 AST（解析 + unify + is_super + extend）

src/eval/
├── mod.rs               # 🆕 求值 dispatcher（纯函数，无状态）
├── builtin.rs           # 🆕 color/math/list/string/meta/modules 内建函数
└── env.rs               # 🆕 环境变量写入/查询（rx 共享 CompileState）
```

### 被替代（不迁移）

- `src/parse_dst/` → `src/css/selector.rs` + `src/eval/`
- `src/evaluate_dst/` → `src/eval/mod.rs`
- `src/shared/` → `src/directive/state.rs`（已存在）
- 旧 `src/directive/{use_,mixin,include,if_,for_,each}.rs` → 统一进 `pipeline.rs` scan_map

## Impact

- **受影响模块**: `src/directive/pipeline.rs` (拆分 → 4 个子文件), 新增 6+ 个源文件
- ** sass-spec 基线**: 从 main 的 8003/12133 (66%) 回退到 rx 的 ~2700/12133 (22%) → 逐步恢复
- **企业验收 (EP)**: main 74/121 (61.2%) → 迁移过程中必然回退，目标最终 > 100/121 (83%)
- **风险**: 中。选择器 + 函数求值两大模块体量大，分 capability 逐个推进可控
- **互不涉及**: 新增文件不影响已有 32 个测试通过的 rx 基础能力
