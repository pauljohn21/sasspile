# AGENTS.md — sasspile 项目规则

## 项目概述

sasspile 是一个纯 Rust 函数式 SCSS 编译器，从零实现，目标是通过 sass-spec 测试套件。

## 架构

```
Source → Lexer → Parser → Evaluator → Serializer → CSS
         (lex/)   (parse/)  (eval/)     (css/)
```

## 强制规则

### 1. Tracing 优先（最高优先级）

**修复任何 bug 前，必须先用 tracing 追踪完整错误链路。**

```bash
# 追踪错误链路
RUST_LOG=info cargo test --lib test_debug_bs_close -- --nocapture

# 完整 span 嵌套
RUST_LOG=debug cargo test --lib test_debug_bs_close -- --nocapture
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

- lib 测试：65 个，必须全通过 `cargo test --lib`
- sass-spec 全量：`cargo test --test sass_spec_full test_sass_spec_full_stats`
- 诊断测试：`cargo test --test cf_diag diag_<subdir> -- --nocapture`
- 修复后必须验证无回归

## 常用命令

```bash
# 编译检查
cargo check

# 运行 lib 测试
cargo test --lib

# 运行 sass-spec 全量统计
RUST_LOG=info cargo test --test sass_spec_full test_sass_spec_full_stats -- --nocapture

# 诊断特定子目录
cargo test --test cf_diag diag_<subdir> -- --nocapture

# 追踪错误链路
RUST_LOG=debug cargo test --lib test_debug_bs_close -- --nocapture
```

## Tracing Span 架构

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

## Git 规范

- 分支：`v2-rewrite-from-scratch`（开发）/ `perf/optimization`（优化）
- 推送：`git push gitee v2-rewrite-from-scratch`
- Commit 格式：`feat: 描述 — 总计 N/M`
- 不主动 commit/push 除非用户要求

## 当前状态

- sass-spec: 2322/10632 (21.9%)
- 65 lib 测试全通过
- 已删除 libsass/non_conformant 目录
- OpenSpec change: v2-rewrite-from-scratch
