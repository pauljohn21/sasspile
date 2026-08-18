## Why

sasspile 当前 `@extend` 实现存在三个层次的严重缺陷，无法通过 sass-spec 中 287 个 extend 相关测试用例：

1. **序列化阶段匹配过于简陋**：`serialize.rs` 使用字符串 `contains` 匹配 extendee，未调用 `selector/extend.rs` 中已有的结构化 `apply_extends_to_list`，导致 compound 选择器统一失败（如 `.a .b {@extend .e}` + `.e.f {x:y}` 应输出 `.a .b.f`）
2. **跨模块 extend 不传播**：`eval_use_rule` 传 `&mut Vec::new()` 给子模块，丢弃了模块内部的 `@extend` 请求，导致 `directives/use/extend/` 全部测试失败
3. **缺少传递性解析与冗余消除**：无 extend 传递图（A→B→C），无循环检测，无 superselector 去重，无 `:is()` 伪类穿透

同时存在两套互不调用的 extend 实现（`serialize.rs` 字符串匹配 vs `selector/extend.rs` 结构化匹配），架构混乱。

## What Changes

- 引入 `ExtensionStore`——集中管理所有 `@extend` 请求的核心数据结构，替代当前散落的 `Vec<ExtendEntry>`
- 实现选择器统一（unification）算法——compound 内 partial 替换、后代选择器 weave 交织
- 实现传递性解析——构建 extend 依赖图，BFS 传播，visited set 检测循环
- 实现模块作用域隔离——每条 extend 携带 `module_id`，基于模块依赖图确定可影响的 CSS 规则
- 实现冗余消除——superselector 检测，移除被包含的选择器
- 实现 placeholder 移除——`%foo` 被扩展后从输出中消失
- 将 extend 应用从 serialize 层移至 eval 层（serialize 只做输出）
- 拆分 `selector/extend.rs`（当前 102 行→预计 800+ 行）为 `selector/extend/` 子模块

## Non-goals

- 不修改 `@use`/`@import`/`@forward` 的模块加载机制（已有 `fix-eval-parser-circular-dep` 变更处理）
- 不实现 `selector.extend()` 内省函数（后续迭代）
- 不实现 `@extend` 的错误消息精确行号定位（后续迭代）
- 不处理 `@import` + `@use` 混合的复杂交互语义的全部边界情况（Phase 5 逐步覆盖）
- 不优化 extend 性能（正确性优先，性能后续迭代）

## Capabilities

### New Capabilities

- `extend-store`: `ExtensionStore` 核心——集中收集、索引、查询所有 `@extend` 请求，支持按 extendee 选择器查找所有 extender
- `selector-unification`: 选择器统一算法——compound 内 partial 替换、complex 后代选择器 weave 交织、`:is()` 伪类穿透
- `extend-transitivity`: 传递性解析——构建 extend 依赖图，BFS 传播 extends，循环检测与截断
- `extend-module-scope`: 模块作用域隔离——每条 extend 携带 `module_id`，基于模块依赖图确定 extend 可影响的 CSS 规则范围
- `extend-redundancy`: 冗余消除——superselector 检测，移除被包含的选择器，placeholder 移除

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

- **`src/selector/extend.rs`** → 拆分为 `src/selector/extend/` 子模块目录
- **`src/selector/extend/mod.rs`** — `ExtensionStore` 结构体 + 公共接口
- **`src/selector/extend/unify.rs`** — compound/complex 选择器统一算法
- **`src/selector/extend/transitive.rs`** — 传递性解析 + 循环检测
- **`src/selector/extend/merge.rs`** — 选择器列表合并 + 冗余消除
- **`src/eval/mod.rs`** — `eval_use_rule` 传递 `&mut extends` 替代 `&mut Vec::new()`；`evaluate` 返回的 `CssTree` 中 extends 已应用
- **`src/serialize.rs`** — 移除 `apply_extends` / `apply_extends_to_rule`，serialize 只做输出
- **`src/eval/func.rs`** — `call_user_function` 中的 `dummy_extends` 改为传递真实 extends
