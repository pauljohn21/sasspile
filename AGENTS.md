# sasslipe-next — AI Agent 指南

本文件指导 AI Agent（如 Claude Code、CatPaw）如何在本项目中高效工作。

## 项目概述

`sasslipe` 是纯 Rust + Tokio 的异步 SCSS 编译器，目标兼容 sass-spec 1306+ 用例（跳过 CSS 4.0 颜色 462 文件）。

**核心架构**：7 阶段 Tokio 异步管道
```
Source → Lex → Parse → Semantic → Transform → Evaluate → Codegen
```

## 技术栈与约束

| 项目 | 规格 |
|------|------|
| Rust Edition | 2024 |
| Toolchain | 1.97 |
| Async Runtime | tokio (features: full) |
| Error Handling | thiserror 2.0 |
| Logging | tracing 0.1 (⛔ 禁止 println!/eprintln!) |
| CLI | clap 4.5 |
| 序列化 | serde, serde_json |

## ⛔ 绝对规则

1. **禁止 Python**：不使用 python3/python/pip，脚本用 rust-script，测试用 #[test]
2. **禁止 println!/eprintln!**：所有代码一律用 tracing 宏（info!/warn!/error!/debug!）
3. **必须使用 tracing span**：涉及跨函数、跨阶段的管道处理必须使用 `tracing::span!`（`info_span!`/`debug_span!`/`trace_span!`）记录执行上下文与耗时，利用 entered/exit 边界追踪调用链，禁止仅用 `event!` 单一日志。字段命名遵循 `stage`、`module`、`id`、`elapsed_ms` 等约定
4. **禁止内联测试**：src/ 保持纯生产代码，所有测试放在 tests/ 目录
5. **单文件 ≤ 400 行**：源码和测试分别计算，超出必须拆分
6. **使用 SSH 推送到 GitHub**

## 工作区结构

```
sasslipe-next/
├── Cargo.toml          # workspace 根（members: sasspile）
├── sasspile/           # SCSS 编译器核心
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs      # 公共 API
│       ├── main.rs     # CLI 入口
│       ├── error.rs    # 错误类型
│       └── pipeline.rs # 管道编排（待实现）
├── openspec/           # OpenSpec 变更与规范
│   └── changes/scss-compiler/
│       ├── proposal.md # 项目提案
│       ├── design.md   # 架构设计
│       └── tasks.md    # 51 个任务清单
```

## 开发工作流

### 1. OpenSpec 驱动开发（推荐）

```bash
# 列出变更
openspec list --json

# 查看变更状态
openspec status --change "scss-compiler" --json

# 获取实施指令
openspec instructions apply --change "scss-compiler" --json

# 实施完成后归档
openspec archive --change "scss-compiler"
```

### 2. 命令行辅助

```bash
# 构建
cargo build -p sasspile

# 测试
cargo test -p sasspile

# Clippy 检查
cargo clippy -p sasspile -- -D warnings

# 文件行数检查（确保 ≤ 400）
find sasspile/src -name "*.rs" -exec wc -l {} \;
```

## 待实现模块（按 Phase）

### Phase 1 — Core Pipeline Foundation
- `source/` — SourceSpan, SourcePosition
- `value/` — Value 枚举、Number、Color、Ops、Coerce、Ser
- `diagnostics/` — Diagnostic、Renderer、Level

### Phase 2 — Lexer
- `lexer/token.rs` — Token 枚举
- `lexer/lexer.rs` — 主词法分析器
- `lexer/sass_syntax.rs` — 缩进语法

### Phase 3 — Parser
- `parser/ast.rs` — AST 定义
- `parser/parser.rs` — 递归下降解析器
- `parser/interpolation.rs` — 插值解析

### Phase 4 — Semantic Analysis
- `semantic/symbol_table.rs` — 作用域栈
- `semantic/module.rs` — @use/@forward 依赖
- `semantic/extends.rs` — @extend 验证

### Phase 5 — Expression Evaluation
- `eval/evaluator.rs` — 求值上下文
- `eval/ops.rs` — 运算符
- `eval/functions.rs` — 函数调用

### Phase 6 — Built-in Modules
- `builtin/sass_color.rs` — sass:color
- `builtin/sass_math.rs` — sass:math
- `builtin/sass_list.rs` — sass:list
- `builtin/sass_map.rs` — sass:map
- `builtin/sass_string.rs` — sass:string
- `builtin/sass_meta.rs` — sass:meta

### Phase 7 — CSS Generation
- `css/generator.rs` — OutputStyle
- `css/rules.rs` — 规则展开
- `css/atrules.rs` — @media/@supports

### Phase 8 — Incremental Compilation
- `incremental/env.rs` — watch channel
- `incremental/depgraph.rs` — 依赖图
- `incremental/cache.rs` — Arc<Value> 缓存

### Phase 9 — Pipeline Orchestration
- `pipeline/mod.rs` — 7-stage Tokio
- `pipeline/backpressure.rs` — bounded channel

## 实施检查清单

每个模块实现时，确认：

- [ ] 文件 ≤ 1000 行
- [ ] mod.rs 仅做 re-export（≤ 200 行）
- [ ] 使用 tracing 宏而非 println!
- [ ] 测试放在 tests/ 目录
- [ ] thiserror 定义错误类型
- [ ] 实现 Clone + Send + Sync + 'static（跨 Task 共享）
- [ ] cargo clippy 无 warning

## 测试策略

1. **单元测试**（tests/ 目录）：每个模块至少 3 个测试用例
2. **Spec 测试**：集成 sass-spec 运行器
3. **CSS4 跳过**：462 个颜色文件在 tests/css4_color_skip.rs 标记

## 常见命令

```bash
# 编译 + 测试完整流程
cargo build -p sasspile && cargo test -p sasspile && cargo clippy -p sasspile -- -D warnings

# 检查行数
find sasspile -name "*.rs" -exec wc -l {} \; | sort -rn | head

# 运行 sasspile CLI
cargo run -p sasspile -- input.scss -o output.css

# watch 模式（待实现）
cargo run -p sasspile -- input.scss --watch
```

## 知识参考

- **架构设计**：`openspec/changes/scss-compiler/design.md`
- **任务清单**：`openspec/changes/scss-compiler/tasks.md`
- **Sass Spec**：sass-spec 仓库（外部）

## 沟通语言

- 对话用**中文**
- 代码、命令、文件名保持英文
