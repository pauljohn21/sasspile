## Why

Sass/SCSS 是 CSS 生态中不可或缺的一级预处理器，而目前 Rust 生态缺少一个**纯异步、函数响应式**的 SCSS 编译器实现。现有的大多数 Sass 绑定都是对 Dart Sass 或 LibSass 的 FFI 封装，存在部署复杂、启动慢、无法热重载等问题。本项目旨在用纯 Rust + Tokio 构建一个**原生异步管道**的 SCSS 编译器，充分利用 Tokio 的并发能力实现并行编译、增量更新和文件监听。

## What Changes

- **新建 `sasslipe` 核心库**：纯 Rust 实现的 SCSS 编译器，无任何外部依赖
- **基于 Tokio 的管道架构**：编译的 7 个阶段（Source → Lex → Parse → Semantic → Transform → Evaluate → Codegen）各自作为独立的 Tokio Task，通过 `mpsc` channel 连接
- **函数响应式状态管理**：使用 `watch` channel 实现变量变更的自动传播和增量重编译
- **Sass spec 兼容性**：以 sass-spec 测试套件（1306+ 用例）为验收标准，确保符合官方 Sass 语义
- **内置模块系统**：实现 `sass:color`, `sass:list`, `sass:map`, `sass:math`, `sass:meta`, `sass:selector`, `sass:string` 全部内置模块
- **CSS 4.0 颜色支持预留**：当前跳过 CSS 4.0 颜色特性（oklch/oklab/lch/lab/display-p3/rec2020 等 462 个文件），但保留接口未来启用

## Capabilities

### New Capabilities
- **`lexer`**: SCSS/Sass 双语法词法分析器，支持 `#{interpolation}` 插值语法
- **`parser`**: 递归下降解析器，产出类型安全的 AST
- **`semantic-analysis`**: 作用域解析、模块依赖图构建、@use/@import/@forward 模块系统
- **`expression-eval`**: 表达式求值引擎，支持所有运算符（算术、逻辑、关系、字符串连接、slash 分隔列表）
- **`value-system`**: 不可变值类型系统（Number/String/Color/List/Map/Calculation/Function）
- **`css-generation`**: CSS 代码生成器，支持嵌套展开、@rule 输出、多种输出格式
- **`builtin-modules`**: 所有 Sass 内置模块的 Rust 实现
- **`pipeline-orchestration`**: Tokio 并行管道编排，支持背压和流式处理
- **`incremental-compile`**: 增量编译，基于文件内容哈希和依赖图实现热重载
- **`diagnostics`**: 带源码位置信息的错误报告系统

### Modified Capabilities

无（全新实现，不修改现有功能）

## Impact

- **新增依赖库**: tokio (async runtime), futures (stream combinators), thiserror (错误处理), tracing (结构化日志)
- **影响范围**: `sasslipe-next/` 整个工作区
- **兼容性目标**: sass-spec 测试套件 1306+ 用例
- **跳过范围**: CSS 4.0 颜色特性（462 个用例），在 `specs/css4-colors/` 中记录跳过理由
