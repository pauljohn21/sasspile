# Proposal: Bootstrap Coverage to 99%

## Why

Bootstrap dist alignment 当前覆盖率 68.10%（tracing 证据：1,704 行缺失 / 5,342 参考行）。深度代码审计发现：覆盖率差距 **并非 4 个独立症状**，而是底层架构违反 rxrust 原则导致的系统性不稳定。

当前 eval pipeline 使用命令式 `for` loop + 手动 LIFO queue 模拟 rxrust 的 `expand` + `scan` 算子，存在 `subscribe-collect` GC 模式、clone 风暴、`'static` 生命周期滥用等反模式。这导致变量 selector cascade 在复杂嵌套中频繁断裂，声明丢失难以诊断。

**根本方案**: 先修复架构债务（消灭 GC 模式、引入真正算子链），再顺势解决 Bootstrap 覆盖差距——而非对四处症状打补丁。

## What Changes

### Layer 1: Architecture Compliance（架构合规）

- **消灭 `subscribe-collect` 模式**: `eval_stream`、`parse_stream` 改用 `.collect::<Vec<_>>()` 算子
- **内联测试迁出**: `serialize/mod.rs`、`builder.rs` 的 `#[cfg(test)]` mod → `tests/` 目录
- **clone 治理**: Work enum 持有 AstNode 为 move 语义，消除 `Arc<EvalContext>` 重复 clone

### Layer 2: rxrust Pipeline Compliance（算子合规）

- **eval pipeline 算子链**: `flat_map(emit_events)` + `scan(initial_frames, fold_frame)` 替代命令式 queue
- **parse pipeline 简化的**: 移除中间 `Arc<Mutex<Vec<Token>>>`，单步 `.collect()` 完成

### Layer 3: Bootstrap-Specific Fixes（覆盖修复）

- **变量 null 过滤修正**: `--bs-*` 自定义属性 null 时 fallback 到 `unset` 而非丢弃整行（~396 行修复）
- **变量 cascade chain walk**: 延迟解析保留 `VarRef`，序列化时最终替换（~200 行修复）
- **Utility API selector 组合**: 修正深层 mixin 嵌套时 `&` 引用与 descendant combinator 的分派（~611 行修复）
- **vendor prefix 自动注入**: 新增 prefixer mixin 覆盖 `file-upload-button`/`column-gap`/`object-fit`/`mask-position`（~80 行修复）

## Capabilities

### Modified Capabilities

- **eval/reactive-pipeline**: 从命令式 queue 模拟升级为真正 rxrust 算子链
- **bootstrap-dist-alignment**: coverage 从 68.10% 提升至 ≥ 99%
- **compiler-architecture**: src/ 零 `#[cfg(test)]` 内联、零 subscribe-collect

### New Capabilities

- **eval/vendor-prefixer**: mixin 层自动 vendor prefix 注入（file-upload-button、column-gap、object-fit、mask-position、transition、appearance）

## Impact

- `src/eval/mod.rs` — 核心重构：消除 `subscribe-collect`，引入 flat_map + scan
- `src/parser/mod.rs` — 简化 collect
- `src/eval/expr.rs` — combine_selectors 分派修正 + VarRef 延迟解析
- `src/eval/builtin.rs` (或新建 `src/eval/prefixer.rs`) — vendor prefix 注入
- `src/serialize/mod.rs` — `# [cfg(test)]` 子移除
- `src/builder.rs` — `#[cfg(test)]` 子移除
- `tests/serialize_test.rs` — 新建，从 src 迁入
- `tests/builder_test.rs` — 新建，从 src 迁入
- 现有测试预期不改变（每层渐进提交，独立通过全量测试）

