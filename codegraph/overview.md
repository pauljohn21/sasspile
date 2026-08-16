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
- [x] sasslipe 核心库（Phase 1-6 已完成）
- [x] sass-spec 集成测试框架（基础骨架）
- [ ] 全量 sass-spec 通过（当前解析通过率 36.4%）
- [ ] CSS 生成器（Phase 7 未开始）
- [ ] 增量编译（Phase 8 未开始）

## 编译管道（7 阶段）

```
Source → Lex → Parse → Semantic → Transform → Evaluate → Codegen
  ↓         ↓       ↓         ↓           ↓          ↓         ↓
 Bytes   Tokens   AST    ResolvedAST  Transformed  Values   CSS AST → String
```

**状态**：Lex → Parse → Semantic → Evaluate 已连通，Codegen 待实现。

## 工作区结构

```
sasslipe-next/
├── hrx/              # ✅ HRX 解析器
├── sasspile/         # ✅ SCSS 编译器核心（Phase 1-6 完成）
│   ├── src/
│   │   ├── source/       # ✅ SourceSpan, SourcePosition
│   │   ├── value/        # ✅ Value 枚举、Number、Color、Ops
│   │   ├── diagnostics/  # ✅ 诊断系统
│   │   ├── lexer/        # ✅ Token + Lexer + Sass 语法
│   │   ├── parser/       # ✅ AST + 递归下降 + 插值
│   │   ├── semantic/     # ✅ 符号表 + 模块 + @extend
│   │   ├── eval/         # ✅ 求值器 + 运算 + 函数
│   │   ├── builtin/      # ✅ sass:color/math/list/map/string/meta
│   │   ├── pipeline.rs   # 🔨 Compiler 骨架
│   │   ├── lib.rs        # 公共 API
│   │   ├── error.rs      # 错误类型
│   │   └── main.rs       # CLI 入口
│   └── tests/            # 集成测试
├── openspec/         # OpenSpec 变更管理
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
| 序列化 | serde, serde_json | 最新 |

## 分支与阶段

| Phase | 内容 | 状态 | 完成日期 |
|-------|------|------|----------|
| 1 | Core Pipeline Foundation (source/value/diagnostics) | ✅ 完成 | 2026-08 |
| 2 | Lexer | ✅ 完成 | 2026-08 |
| 3 | Parser (AST + 递归下降 + 插值 + @规则 + 选择器) | ✅ 完成 | 2026-08 |
| 4 | Semantic Analysis (符号表/模块/@extend) | ✅ 完成 | 2026-08 |
| 5 | Expression Evaluation | ✅ 完成 | 2026-08 |
| 6 | Built-in Modules (sass:color/math/list/map/string/meta) | ✅ 完成 | 2026-08 |
| 7 | CSS Generation | ❌ 待开发 | - |
| 8 | Incremental Compilation | ❌ 待开发 | - |
| 9 | Pipeline Orchestration | 🔨 骨架存在 | - |
| 10 | CSS4 Colors | ❌ 后续阶段 | - |
| 11 | Testing & Sass Spec | 🔨 进行中 (36.4%) | - |
| 12 | Documentation & Release | ❌ 待开始 | - |

## sass-spec 进度

| 指标 | 数值 |
|------|------|
| 总用例 | 1306 |
| 解析通过 | 475 |
| 通过率 | 36.4% |
| CSS4 颜色跳过 | 462 |

### 已知失败模式（优先级从高到低）

1. **`and`/`or` 逻辑运算符缺失** — 影响大量条件表达式
2. `@else`/`@else if` 解析分支
3. `@if` 条件中含 `and` 组合
4. 大括号追踪（嵌套边界）
5. `@extend` 多行选择器

## 兼容性目标

- sass-spec non-color 用例：100% 通过（当前进行中）
- CSS 4.0 颜色特性（462 文件）：本期跳过，保留接口
