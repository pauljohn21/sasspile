# 项目概览

## 目标

构建纯 Rust + Tokio 的 SCSS 编译器 `sasslipe`，兼容 sass-spec 测试套件（1306+ 用例，跳过 CSS 4.0 颜色 462 文件）。

## 核心原则

1. **纯异步管道**：每个编译阶段是独立 Tokio Task，通过 `mpsc` channel 连接
2. **函数响应式**：不可变数据 + `watch` channel 实现增量更新
3. **单文件精简**：每文件 ≤ 400 行，职责单一
4. **零外部依赖**：除 tokio/tracing/futures/thiserror 外无其他依赖
5. **Sass 语义兼容**：以 sass-spec 为验收标准

## 当前状态

- [x] HRX 解析器（已完成，用于读取 sass-spec 测试用例）
- [ ] sasslipe 核心库（规划中）
- [ ] sass-spec 集成测试框架（待开发）

## 编译管道（7 阶段）

```
Source → Lex → Parse → Semantic → Transform → Evaluate → Codegen
  ↓         ↓       ↓         ↓           ↓          ↓         ↓
 Bytes   Tokens   AST    ResolvedAST  Transformed  Values   CSS AST → String
```

## 工作区结构

```
sasslipe-next/
├── hrx/              # HRX 解析器（已实现）
│   ├── src/          # parser.rs, models.rs, error.rs, writer.rs
│   └── tests/        # 集成测试
├── openspec/         # OpenSpec 变更管理
│   ├── changes/scss-compiler/  # SCSS 编译器变更提案
│   │   ├── proposal.md
│   │   ├── design.md
│   │   └── tasks.md  # 分阶段任务清单（Phase 1-12）
│   └── specs/        # 能力规范
└── codegraph/        # 项目知识图谱（本目录）
```

## 技术栈

| 组件 | 选择 | 版本 |
|------|------|------|
| Rust Edition | 2024 | 1.97+ |
| Async Runtime | tokio | 最新 |
| 错误处理 | thiserror | 2.0 |
| 日志 | tracing | 0.1 |
| 命令行 | clap | 4.5 |
| 测试 | 内置 #[test] | - |

## 分支与阶段

| Phase | 内容 | 状态 |
|-------|------|------|
| 1 | Core Pipeline Foundation | 待开始 |
| 2 | Lexer | 待开始 |
| 3 | Parser | 待开始 |
| 4 | Semantic Analysis | 待开始 |
| 5 | Expression Evaluation | 待开始 |
| 6 | Built-in Modules | 待开始 |
| 7 | CSS Generation | 待开始 |
| 8 | Incremental Compilation | 待开始 |
| 9 | Pipeline Orchestration | 待开始 |
| 10 | CSS4 Colors | 后续阶段 |
| 11 | Testing & Sass Spec | 待开始 |
| 12 | Documentation & Release | 待开始 |

## 兼容性目标

- sass-spec non-color 用例：100% 通过
- CSS 4.0 颜色特性（462 文件）：本期跳过，保留接口
