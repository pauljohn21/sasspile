# scss-rs

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

纯 Rust SCSS 编译器，从零重写，函数式架构。

> **v0.1.0** — 骨架搭建完成，类型状态机管线 + const 静态表 dispatch — sass-spec 279/5362 (5%)，ep_full 16/121 (13%)，bs_spec 15/15 (100%)。

scss-rs 是 sasspile 的全新重写版本，采用更干净的函数式架构：

- **类型状态机管线**：`Source → Lexed → Parsed → Evaluated → Serialized`，TryFrom 转换
- **零 proc-macro**：内建函数用 const 静态表做单一数据源，编译期验证
- **零外部依赖**：纯 std 实现（tracing 为可选 feature）
- **move 语义 Env**：`enter_scope`/`exit_scope` 管理作用域，builder 模式链式调用

```rust
use scss_rs::{compile_expanded, OutputStyle};

let css = compile_expanded("a { color: red; }")?;
// => "a {\n  color: red;\n}\n"
```

## 架构

```
Source { text, base_path, load_paths }
   │
   ▼  TryFrom<Source> for Lexed
Lexed { tokens, base_path, load_paths }
   │
   ▼  TryFrom<Lexed> for Parsed
Parsed { ast, base_path, load_paths }
   │
   ▼  TryFrom<Parsed> for Evaluated
Evaluated { nodes: Vec<CssNode> }
   │
   ▼  Serialized::from_nodes()
Serialized { css: String }
```

## 快速开始

```rust
use scss_rs::{compile, compile_expanded, compile_file, OutputStyle};

// 字符串编译
let css = compile("a { color: red; }", OutputStyle::Expanded)?;

// 简写——展开模式
let css = compile_expanded("a { color: red; }")?;

// 文件编译
let css = compile_file(&path, OutputStyle::Expanded)?;
```

## 支持的功能

### 已实现（骨架）

- 变量 `$var` + 赋值
- 嵌套规则 + 父选择器 `&`
- `@mixin` / `@include` / `@content`
- `@function` / `@return`
- `@if` / `@else if` / `@else`
- `@for` / `@each` / `@while`
- `@use` / `@forward` / `@import`（骨架）
- `@extend` / `@at-root` / `@warn` / `@debug` / `@error`
- 数学/字符串/列表/Map/meta 内建函数（基础版）
- 展开/压缩序列化

### 开发中（align-sasspile 变更）

- 完整表达式解析器
- 模块系统（文件解析 + 缓存）
- @extend 选择器引擎
- 颜色系统（RGB/HSL/HWB）
- compressed 序列化

## 测试

```bash
# 编译测试
cargo test --test compile_test     # 6 个

# 兼容性测试
cargo test --test bs_spec          # 15 个 Bootstrap 测试
cargo test --test ep_full          # Element Plus 全量

# sass-spec
cargo test --test sass_spec_full  # 全量统计
```

## 项目结构

```
src/
├── lib.rs              # 公开 API + 管线入口
├── error.rs            # SassError
├── source.rs           # Source 类型
├── lex/                # 词法分析
├── parse/              # 语法分析 + AST
├── eval/               # 求值器 + Env + 内建函数
│   └── builtin/        # const 静态表 dispatch
└── css/                # 序列化器 + CssNode
```

## 设计文档

- `docs/ARCHITECTURE.md` — 架构总览
- `docs/MODULE_BOUNDARIES.md` — 模块边界与依赖方向
- `docs/DATA_FLOW.md` — 数据流与类型定义
- `openspec/` — OpenSpec 变更管理

## 许可证

MIT
