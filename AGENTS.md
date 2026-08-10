# AGENTS.md — sasspile 项目规则

## 项目概述

sasspile 是一个纯 Rust 函数式 SCSS 编译器，从零实现，目标是通过 sass-spec 测试套件。

## 架构

```
Source → Lexer → Parser → Evaluator → Serializer → CSS
         (lex/)   (parse/)  (eval/)     (css/)
```

> **查找函数/类型/概念在哪个文件？** 见 [`docs/CODE_INDEX.md`](docs/CODE_INDEX.md)。

## 强制规则

### 1. Tracing 优先（最高优先级）

**修复任何 bug 前，必须先用 tracing 追踪完整错误链路。**

```bash
# 追踪错误链路
RUST_LOG=info cargo test --lib test_debug_bs_close -- --nocapture

# 完整 span 嵌套
RUST_LOG=debug cargo test --lib test_debug_bs_close -- --nocapture

# Per-target 过滤（只看颜色相关 events）
RUST_LOG="sasspile::color=debug" cargo test --lib -- --nocapture

# 组合多个 target
RUST_LOG="sasspile::color=trace,sasspile::extend=info" cargo test --lib -- --nocapture
```

详见 `.claude/skills/tracing-debug/SKILL.md`。

### 2. Rust Edition 2024, Toolchain 1.97

- 新代码必须使用 edition 2024 语法
- Cargo.toml 中 `edition = "2024"`

### 3. 禁止 Python

- 不得使用 python3/python/pip 或创建 .py 文件
- 脚本用 rust-script，表达式用 rust-script -e
- 测试用 `#[test]`，依赖用 Cargo.toml

### 4. 代码规范

- 公开 API 必须有 `///` 文档注释
- 模块用 `//!` 文档注释
- 禁止 `unwrap()` 生产代码——用 `?` / `expect()` / `unwrap_or()`
- 关键函数用 `#[instrument]` 或手动 span 追踪

### 5. 测试规范

- lib 测试：77 个，必须全通过 `cargo test --lib`
- diff 测试：5 个，`cargo test --test common_test`
- sass-spec 全量：`cargo test --test sass_spec_full test_sass_spec_full_stats`
- 诊断测试：`cargo test --test cf_diag diag_<subdir> -- --nocapture`
- 最小化工具：`cargo test --test minimize minimize_<subdir>_error -- --nocapture`
- 修复后必须验证无回归

## 常用命令

```bash
# 编译检查
cargo check

# 运行 lib 测试（77 个）
cargo test --lib

# 运行 diff 测试（5 个）
cargo test --test common_test

# 运行 sass-spec 全量统计
RUST_LOG=info cargo test --test sass_spec_full test_sass_spec_full_stats -- --nocapture

# 诊断特定子目录（集成 CSS 逐行 diff）
cargo test --test cf_diag diag_<subdir> -- --nocapture

# CSS diff 详情模式
RUST_LOG="cssdiff=debug" cargo test --test cf_diag diag_<subdir> -- --nocapture

# sass-spec 最小化工具（delta debugging）
RUST_LOG="minimize=info" cargo test --test minimize minimize_color_error -- --nocapture

# 追踪错误链路
RUST_LOG=debug cargo test --lib test_debug_bs_close -- --nocapture

# Per-target 过滤
RUST_LOG="sasspile::color=trace" cargo test --lib -- --nocapture
RUST_LOG="sasspile::extend=debug,sasspile::binop=trace" cargo test --lib -- --nocapture
```

## Tracing 架构

### Span 层级（结构追踪）

```
eval_nodes → eval_node_item → eval_node
  ├── eval_rule (selector)
  ├── eval_for (var, inclusive)
  ├── eval_each (n_vars)
  ├── eval_include (name, n_args)
  └── eval_value → call_function
      ├── call_builtin (name, n_args)
      ├── call_module_function (name)
      └── call_user_function (n_params, n_args)
```

文件加载：
```
load_module (path, depth) → resolve_file (url)
```

@extend 后处理：
```
apply_extends (n_extends) → 递归遍历 CSS 树
```

### Event Targets（值快照）

> **Span = 结构（WHERE），Event = 值（WHAT）**

| Target | Level | 场景 |
|--------|-------|------|
| `sasspile::color` | trace | 颜色转换函数输入/输出（hsl_to_rgb, rgb_to_hsl, hwb_to_rgb） |
| `sasspile::color` | debug | 颜色 builtin 函数入口/结果（darken, lighten, mix, rgba） |
| `sasspile::extend` | info | @extend 匹配成功 |
| `sasspile::extend` | debug | 选择器替换细节（占位符替换、继承者添加） |
| `sasspile::binop` | trace | 二元运算操作数值 + 结果 |
| `cssdiff` | info | CSS diff 检测摘要 |
| `cssdiff` | debug | 行级差异详情 |
| `minimize` | info | 最小化轮次摘要 |
| `minimize` | debug | 每次移除尝试 |

### 调试工具

- **CSS Diff 模块** (`tests/common/mod.rs`)：逐行对比期望 vs 实际 CSS，分类统计（content_diff/missing_output/extra_output）
- **sass-spec 最小化工具** (`tests/minimize.rs`)：Delta debugging 自动最小化失败用例到最小复现代码
- **Node::to_scss()** (`src/parse/ast_impl.rs`)：AST → SCSS 序列化，支持最小化工具

### 源文件结构

全部源文件 ≤ 500 行（最大 `eval/builtin.rs` 459 行）。

```
src/
├── lib.rs           (300)  公共 API + init_tracing
├── main.rs          (36)
├── error.rs         (77)
├── css/
│   ├── mod.rs       (248)  Serializer
│   └── node.rs      (73)   CssNode
├── lex/
│   ├── mod.rs       (404)  Lexer + Iterator impl
│   └── token.rs     (131)  Token 定义
├── parse/
│   ├── mod.rs       (75)   Parser 结构 + parse() 入口
│   ├── nodes.rs     (348)  节点解析 + 参数解析
│   ├── at_rules.rs  (364)  @规则解析
│   ├── expr.rs      (350)  Pratt 表达式 + 数值/颜色解析
│   ├── ast.rs       (310)  AST 类型定义
│   └── ast_impl.rs  (201)  Display + to_scss 实现
├── eval/
│   ├── mod.rs       (319)  Env + Evaluator + eval_nodes/eval_node
│   ├── rule.rs      (117)  eval_rule + combine_selectors
│   ├── value.rs     (356)  eval_value + binop + 算术运算
│   ├── control_flow.rs(111)eval_if/for/each/while
│   ├── mixin.rs     (138)  eval_include + call_function
│   ├── extend.rs    (72)   apply_extends
│   ├── module.rs    (169)  resolve_file + load_module
│   ├── color.rs     (241)  颜色转换 + builtin 颜色函数
│   ├── builtin.rs   (459)  call_builtin 分派入口
│   └── builtin/
│       ├── list.rs  (154)  list 内建函数
│       └── selector.rs(75) selector 内建函数
└── stage/                  管线阶段类型
```

## Git 规范

- 分支：`v2-rewrite-from-scratch`（开发）/ `perf/optimization`（优化）
- 推送：`git push gitee v2-rewrite-from-scratch`
- Commit 格式：`feat: 描述 — 总计 N/M`
- 不主动 commit/push 除非用户要求

## 当前状态

- sass-spec: 2322/10632 (21.9%)
- 77 lib 测试 + 5 diff 测试全通过
- 全部源文件 ≤ 500 行（文件拆分完成）
- 调试工具链：CSS diff + 最小化 + 值快照 events
- 已删除 libsass/non_conformant 目录
- OpenSpec change: v2-rewrite-from-scratch
