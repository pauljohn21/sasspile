## Why

sasspile 经过多次 AI 迭代后，代码库中积累了大量 GC 思维（命令式/过程式）产物，严重违反 rxrust 响应式编程哲学。具体表现为：用 `while`/`for` + `mut result` 的文本手术替代 `map`/`filter` 算子；模块系统 (`module_system.rs` / `import_resolver.rs`) 完全绕过 rxrust 管线；doc comment 中充斥着 "Flux → rxrust 转译" 等错误表述；`if/return` 控制流链和 `clone()` 泄漏遍布核心路径。本 change 旨在建立一套强制性的 spec 约束，确保后续 AI 迭代严格遵守 Rust ownership 三态范式。

## What Changes

- **新增 Spec 约束文件**：在 `specs/` 下创建 `rxrust-ownership/` 能力域，定义"什么算违规"
- **清理错误术语**：所有 doc comment 中 "Flux 思维" / "Flux → rxrust 转译" 等表述替换为 Rust ownership 三态语言
- **过程式代码标记**：在现有代码中识别并标记 GC 思维反模式，提供修复指南
- **AI 行为约束**：spec 中明确列出 AI 写代码时的"绝对禁止项"和"必须遵守项"
- **验收标准**：建立代码审核 checklist，每次 AI 产出必须通过才能提交

## Capabilities

### New Capabilities
- `rxrust-ownership`: 核心响应式编程约束 — 定义所有权三态、算子选择表、禁止模式清单
- `reactive-dataflow`: 数据流规范 — 管线入口/中间/终端的标准模式、scan_map 作为唯一状态栖息地
- `no-gc-patterns`: GC 思维检测与禁止 — 列出所有禁止的命令式模式及其正确替代
- `module-system-rx`: 模块系统响应式化约束 — @use/@forward 如何在管线内（而非绕过管线）运作
- `tracing-span`: tracing span 强制规范 — 跨函数/跨阶段必须使用 span 而非 println!

### Modified Capabilities
- 无（这是全新约束体系，不修改已有 spec 的需求）

## Impact

- **影响文件**：`src/lib.rs`, `src/directive/mod.rs`, `src/directive/pipeline.rs`, `src/directive/parse.rs`, `src/directive/module_system.rs`, `src/directive/import_resolver.rs`, `src/directive/ops.rs`, `src/directive/while_ops.rs`, `src/eval/mod.rs`, `src/directive/state.rs`, `src/css/node.rs`
- **影响范围**：所有后续 AI 编码会话的行为约束
- **依赖**：rxrust 1.0.0-rc.5 源码
- **风险**：spec 是行为约束，不直接改代码；代码重构是后续 change 的事
