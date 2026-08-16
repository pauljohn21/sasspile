# sasspile

**纯 Rust + Tokio 异步 SCSS 编译器**，兼容 [Sass 规范](https://github.com/sass/sass-spec)。

[![crates.io](https://img.shields.io/crates/v/sasspile.svg)](https://crates.io/crates/sasspile)
[![docs.rs](https://img.shields.io/docsrs/sasspile)](https://docs.rs/sasspile)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](./LICENSE)
[![Rust Edition 2024](https://img.shields.io/badge/rust-2024-orange.svg)](https://blog.rust-lang.org/2024/02/19/Rust-2024.html)
[![Rust Toolchain 1.97](https://img.shields.io/badge/toolchain-1.97-blue.svg)](https://blog.rust-lang.org/2026/04/02/Rust-1.97.0.html)
[![Downloads](https://img.shields.io/crates/d/sasspile.svg)](https://crates.io/crates/sasspile)

## 目录

- [概览](#概览)
- [架构](#架构)
- [安装](#安装)
- [使用](#使用)
- [功能](#功能)
- [兼容性](#兼容性)
- [测试](#测试)
- [项目结构](#项目结构)
- [设计原则](#设计原则)
- [路线图](#路线图)
- [贡献](#贡献)
- [许可](#许可)

## 概览

sasspile 是从零开始用 Rust 编写的 SCSS 编译器，围绕 **7 阶段 Tokio 异步管道**构建。它在利用 Rust 类型系统和异步运行时实现安全、高性能编译的同时，致力于与 [Sass 规范](https://sass-lang.com/documentation)广泛兼容。

**当前状态**：核心管道已完成 — 词法分析、语法分析、语义分析、表达式求值、内置模块（color/math/list/map/string/meta）、CSS 生成、CSS4 颜色空间、增量编译、管道编排、Sass spec 测试集成。

## 架构

```
Source → Lex → Parse → Semantic → Transform → Evaluate → Codegen
  │         │       │         │          │          │         │
  └─────────┴───────┴─────────┴──────────┴──────────┴─────────┘
                    Tokio Tasks + mpsc Channels
```

每个阶段都是独立的 Tokio 任务，通过有界 `mpsc` 通道连接。不可变的 `Value` 类型流经管道，`watch` 通道传播变量变更以支持增量重编译。

### 模块结构

| 模块 | 用途 |
|------|------|
| `lexer` | 词法分析、插值解析、缩进语法 |
| `parser` | 递归下降解析器、AST 构建、错误恢复 |
| `semantic` | 符号表、依赖解析、`@extend` 验证 |
| `eval` | 表达式求值、运算符、函数调度 |
| `builtin` | 内置 Sass 模块：`sass:color`、`sass:math`、`sass:list`、`sass:map`、`sass:string`、`sass:meta` |
| `css` | CSS 输出生成、规则展开、at-rule 处理 |
| `value` | 值系统：数字、颜色、字符串、Map、List |
| `color` | CSS4 颜色空间：Oklab、Oklch、HWB |
| `pipeline` | 7 阶段 Tokio 编排 + 背压控制 |
| `incremental` | 反应式环境、依赖图、缓存、传播 |
| `diagnostics` | 带源代码片段的错误报告 |

## 安装

### 从 crates.io

```bash
cargo install sasspile
```

### 从源码

```bash
git clone git@github.com:pauljohn21/sasspile-next.git
cd sasspile-next
cargo build --release -p sasspile
```

### 依赖要求

- Rust 1.97+ (edition 2024)
- Tokio (features: full)

## 使用

### CLI

```bash
# 编译单个文件
sasspile input.scss -o output.css



### 作为库使用

```rust
use sasspile::Compiler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compiler = Compiler::new();
    let css = compiler.compile("$color: red; .foo { color: $color; }").await?;
    println!("{css}");
    Ok(())
}
```

### Pipeline（高级）

```rust
use sasspile::{Pipeline, PipelineInput};
use sasspile::css::OutputStyle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipeline = Pipeline::new();

    let input = PipelineInput {
        path: "input.scss".to_string(),
        source: ".foo { color: blue; }".to_string(),
    };

    let output = pipeline.compile_one(input).await?;
    println!("{output:?}");
    Ok(())
}
```

### 批量编译（并发）

```rust
use sasspile::{Pipeline, PipelineInput};
use sasspile::css::OutputStyle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipeline = Pipeline::new();

    let inputs = vec![
        PipelineInput { path: "a.scss".to_string(), source: ".a { color: red; }".to_string() },
        PipelineInput { path: "b.scss".to_string(), source: ".b { color: blue; }".to_string() },
    ];

    let results = pipeline.compile_batch(inputs, OutputStyle::Expanded).await;

    for result in results {
        match result {
            Ok(output) => println!("{}: {} bytes", output.path, output.css.len()),
            Err(e) => eprintln!("Error: {e}"),
        }
    }
    Ok(())
}
```

### 增量编译

```rust
use sasspile::incremental::{ReactiveEnv, DependencyGraph, SpanCache};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 反应式环境：变量变更触发下游更新
    let env = ReactiveEnv::new();
    env.set_var("theme", "dark");

    // 跟踪节点间的依赖关系
    let graph = DependencyGraph::new();

    // 缓存编译后的 span
    let cache = SpanCache::new();

    // 通过管道传播变更
    // (使用 PropagateMsg::VarChanged, BatchChanged, 或 SourceEdited)
    Ok(())
}
```

## 功能

### 已实现

- **词法分析**：标识符、数字、字符串、运算符、插值 `#{}`
- **语法分析**：规则、声明、at-rule、嵌套、选择器
- **语义分析**：符号表、模块依赖、`@extend`
- **求值**：算术、比较、字符串操作、列表/Map 访问
- **内置模块**：`sass:color`、`sass:math`、`sass:list`、`sass:map`、`sass:string`、`sass:meta`
- **CSS 生成**：expanded/compressed 输出、嵌套规则、at-rules
- **CSS4 颜色**：Oklab、Oklch、HWB、`color-mix()`、相对颜色语法
- **管道**：7 阶段 Tokio 异步管道 + 背压
- **增量编译**：反应式环境、依赖图、span 缓存、变更传播
- **Sass 规范**：HRX 加载器、spec 运行器、CSS4 颜色跳过列表

### CSS4 颜色空间

| 颜色空间 | 状态 |
|----------|------|
| sRGB (hex, rgb(), hsl()) | ✅ |
| Oklab / Oklch | ✅ |
| HWB | ✅ |
| `color-mix()` | ✅ |
| 相对颜色语法 | ✅ |
| `light-dark()` | ✅ 通过 `color-mix()` |

## 兼容性

| 测试套件 | 通过率 | 说明 |
|----------|--------|---------|
| **Bootstrap** | **100%** (99/99) | 完整 Bootstrap 5 SCSS 编译基线 |
| **Element Plus** | **100%** (145/145) | 核心规范兼容性 |
| **sass-spec (parse)** | 持续追踪 | 解析器兼容性 |

> 完整兼容性数据通过 `cargo test -p sasspile --test sass_spec_parse` 获取实时报告。

## 测试

```bash
# 运行所有单元测试
cargo test -p sasspile --lib

# 运行集成测试（spec runner）
cargo test -p sasspile --test spec_runner

# 运行全部测试
cargo test -p sasspile --lib --tests

# Clippy 检查（零警告）
cargo clippy -p sasspile -- -D warnings
```

## 项目结构

```
sasslipe-next/
├── sasspile/              # SCSS 编译器核心 crate
│   ├── Cargo.toml
│   ├── examples/          # 示例代码
│   │   ├── basic_compile.rs
│   │   ├── concurrent_compile.rs
│   │   ├── incremental_compile.rs
│   │   └── trace_parse.rs
│   ├── src/               # 源码（每文件 ≤ 400 行）
│   │   ├── lib.rs         # 公共 API
│   │   ├── main.rs        # CLI 入口
│   │   ├── error.rs       # 错误类型
│   │   ├── lexer/         # 词法分析
│   │   ├── parser/        # 语法分析
│   │   ├── semantic/      # 语义分析
│   │   ├── eval/          # 表达式求值
│   │   ├── builtin/       # 内置模块
│   │   ├── css/           # CSS 生成
│   │   ├── value/         # 值系统
│   │   ├── color/         # CSS4 颜色
│   │   ├── pipeline/      # 管道编排
│   │   ├── incremental/   # 增量编译
│   │   └── diagnostics/   # 诊断报告
│   └── tests/             # 集成测试
├── bs/                    # Bootstrap SCSS（子模块）
├── ep/                    # Element Plus SCSS（子模块）
```

## 设计原则

1. **零 `println!`** — 全部日志通过 `tracing` 宏
2. **不可变数据** — 值使用 `Arc` 以便在任务间廉价共享
3. **模块化文件** — 单文件 ≤ 400 行
4. **异步优先** — 管道全流程 Tokio 任务
5. **零 unsafe** — 纯安全 Rust



## 贡献

欢迎贡献！请确保：

- `cargo build -p sasspile` 通过
- `cargo test -p sasspile` 通过
- `cargo clippy -p sasspile -- -D warnings` 无警告
- 所有日志使用 `tracing` 宏（禁止 `println!`/`eprintln!`）
- 新测试放在 `tests/` 目录（不要内联）
- 单文件 ≤ 400 行

## 许可

双许可：[MIT](./LICENSE-MIT) 或 [Apache-2.0](./LICENSE-APACHE)

## 致谢

- [Sass specification](https://sass-lang.com/documentation) — 权威参考
- [sass-spec](https://github.com/sass/sass-spec) — 官方测试套件
- [Bootstrap](https://github.com/twbs/bootstrap) — 真实世界 SCSS 基线
