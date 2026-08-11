# AGENTS.md — sasspile 项目规则

## 项目概述

sasspile 是一个纯 Rust 函数式 SCSS 编译器，从零实现，目标是通过 sass-spec 测试套件。

## 架构

```
Source → Lexer → Parser → Evaluator → Serializer → CSS
         (lex/)   (parse/)  (eval/)     (css/)
```

> **查找函数/类型/概念在哪个文件？** 见 [`docs/CODE_INDEX.md`](docs/CODE_INDEX.md)。
> **动态查询调用关系/源码/影响范围？** 用 CodeGraph（见下方 [CodeGraph 代码导航](#codegraph-代码导航)）。

## CodeGraph 代码导航

项目已集成 [CodeGraph](https://github.com/the-codegraph-project/codegraph)——一个基于 SQLite 的代码知识图谱，索引了全部 56 个源文件的 700 个符号和 2,792 条调用边。

> **何时用 CodeGraph vs `docs/CODE_INDEX.md`？**
> - CODE_INDEX.md = 静态速查表（函数→文件映射），快速定位。
> - CodeGraph = **动态查询**（调用者/被调用者/影响分析/源码查看），支持追溯和影响分析。
> - **优先使用 CodeGraph** 进行调用链路追踪和修改影响分析，避免手动逐文件读取。

### 索引管理

```bash
# 查看索引状态（文件数/节点数/边数/是否最新）
codegraph status

# 修改源码后同步增量索引
codegraph sync

# 从零重建完整索引（大改动后推荐）
codegraph index -v
```

### 查询命令

```bash
# 搜索符号（按名称模糊匹配）
codegraph query eval_value
codegraph query -k function "parse_"     # 按类型过滤（function/method/struct/enum/...）
codegraph query -l 20 "color"            # 增加结果上限

# 查看符号源码 + 调用者/被调用者链路（一站式，无需 Read 文件）
codegraph node eval_node                 # 直接输出源码 + Calls→ / Called-by← 链路
codegraph node -f src/eval/mod.rs       # 文件模式：带行号读取文件 + 依赖列表
codegraph node -f src/eval/mod.rs --offset 250 --limit 30  # 读取指定行范围

# 查找调用者（谁调了这个函数？）
codegraph callers apply_extends          # 修改函数前先看影响面

# 查找被调用者（这个函数调了谁？）
codegraph callees eval_node

# 影响分析（修改某符号会影响哪些代码？——推荐修改前运行）
codegraph impact eval_value              # 默认深度 2
codegraph impact -d 3 eval_value         # 增加深度

# 一次性探索某个领域（源码 + 调用路径）
codegraph explore "color conversion functions"
codegraph explore "module loading and file resolution"

# 查看项目文件结构（含符号数量）
codegraph files

# 查找受影响的测试文件（修改源码后跑哪些测试？）
codegraph affected src/eval/color.rs src/eval/value.rs
```

### 典型工作流

#### 1. 修 bug 前的链路追踪

```bash
# Step 1: 找到目标函数的位置和源码
codegraph node eval_value

# Step 2: 查看谁调了它（上游影响面）
codegraph callers eval_value

# Step 3: 查看它调了谁（下游依赖）
codegraph callees eval_value

# Step 4: 完整影响分析
codegraph impact eval_value
```

#### 2. 重构前的影响评估

```bash
# 评估修改某函数的波及范围
codegraph impact -d 3 call_builtin

# 找到需要回归测试的测试文件
codegraph affected src/eval/builtin.rs
```

#### 3. 探索不熟悉的代码区域

```bash
# 用自然语言描述探索目标
codegraph explore "extend selector inheritance"
# 输出：相关符号源码 + 调用路径，一站式获取上下文
```

### 索引统计（2026-08-11）

| 指标 | 数值 |
|------|------|
| 文件数 | 56 |
| 节点数 | 700 |
| 边数 | 2,792 |
| 函数 | 201 |
| 方法 | 182 |
| 枚举成员 | 123 |
| 结构体 | 22 |
| 枚举 | 11 |

## 强制规则

### 1. Tracing 优先（最高优先级）

**修复任何 bug 前，必须先用 tracing 追踪完整错误链路。**

```bash
# 追踪错误链路
RUST_LOG=info cargo test --test compile_test test_debug_bs_close -- --nocapture

# 完整 span 嵌套
RUST_LOG=debug cargo test --test compile_test test_debug_bs_close -- --nocapture

# Per-target 过滤（只看颜色相关 events）
RUST_LOG="sasspile::color=debug" cargo test --test compile_test -- --nocapture

# 组合多个 target
RUST_LOG="sasspile::color=trace,sasspile::extend=info" cargo test --test compile_test -- --nocapture
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
- **禁止 `eprintln!`/`println!`**——所有代码（含 src/ 和 tests/）一律用 `tracing` 宏
- **禁止 `#[cfg(test)]` 内联测试**——所有测试放在 `tests/` 目录，`src/` 保持纯生产代码

### 5. 测试规范

**物理隔离原则**：所有测试代码放在 `tests/` 目录，`src/` 中不包含 `#[cfg(test)]` 块。

**Tracing 原则**：测试和 CLI 中禁止使用 `eprintln!`/`println!`，一律用 `tracing` 宏（`info!`/`warn!`/`error!`/`debug!`）。

- compile 测试：28 个，`cargo test --test compile_test`（编译管线端到端测试）
- stage 测试：10 个，`cargo test --test stage_test`（阶段类型 + CSS Serializer 单元测试）
- ast 测试：8 个，`cargo test --test ast_test`（AST Display 测试）
- diff 测试：5 个，`cargo test --test common_test`（CSS diff 工具测试）
- Bootstrap 全量：`cargo test --test bs_spec -- --nocapture`（15 个测试，`bootstrap.scss` 全量编译通过）
- Element Plus 全量：`cargo test --test ep_full -- --nocapture`（121/121 100% 通过）
- sass-spec 全量：`cargo test --test sass_spec_full test_sass_spec_full_stats`
- 诊断测试：`cargo test --test cf_diag diag_<subdir> -- --nocapture`
- 最小化工具：`cargo test --test minimize minimize_<subdir>_error -- --nocapture`
- 修复后必须验证无回归

## 常用命令

```bash
# 编译检查
cargo check

# 运行 compile 测试（28 个，秒级）
cargo test --test compile_test

# 运行 stage 测试（10 个，秒级）
cargo test --test stage_test

# 运行 ast 测试（8 个，秒级）
cargo test --test ast_test

# 运行 diff 测试（5 个）
cargo test --test common_test

# 运行 Bootstrap 全量编译验证（15 个测试）
cargo test --test bs_spec -- --nocapture

# 运行 Element Plus 全量编译验证（121 个文件）
cargo test --test ep_full -- --nocapture

# 运行 sass-spec 全量统计
RUST_LOG=info cargo test --test sass_spec_full test_sass_spec_full_stats -- --nocapture

# 诊断特定子目录（集成 CSS 逐行 diff）
cargo test --test cf_diag diag_<subdir> -- --nocapture

# CSS diff 详情模式
RUST_LOG="cssdiff=debug" cargo test --test cf_diag diag_<subdir> -- --nocapture

# sass-spec 最小化工具（delta debugging）
RUST_LOG="minimize=info" cargo test --test minimize minimize_color_error -- --nocapture

# 追踪错误链路
RUST_LOG=debug cargo test --test compile_test test_debug_bs_close -- --nocapture

# Per-target 过滤
RUST_LOG="sasspile::color=trace" cargo test --test compile_test -- --nocapture
RUST_LOG="sasspile::extend=debug,sasspile::binop=trace" cargo test --test compile_test -- --nocapture
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
load_module (path, depth) → resolve_file (url, load_paths)
```

@return 控制流：
```
eval_node(Return) → CssNode::Return(Value) → call_user_function 捕获
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

最大源文件 `parse/expr.rs` 623 行，3 个文件超过 500 行（待拆分）。

```
src/
├── lib.rs            (283)  公共 API + init_tracing（无内联测试）
├── main.rs           (29)
├── error.rs          (80)
├── css/
│   ├── mod.rs        (358)  Serializer
│   └── node.rs       (88)   CssNode（含 Return 变体）
├── lex/
│   ├── mod.rs        (492)  Lexer + Iterator impl
│   └── token.rs      (131)  Token 定义
├── parse/
│   ├── mod.rs        (92)   Parser 结构 + parse() 入口
│   ├── nodes.rs      (488)  节点解析 + 参数解析
│   ├── at_rules.rs   (451)  @规则解析
│   ├── expr.rs       (623)  Pratt 表达式 + 数值/颜色解析
│   ├── ast.rs        (375)  AST 类型定义
│   └── ast_impl.rs   (281)  Display + to_scss 实现
├── eval/
│   ├── mod.rs        (454)  Env + Evaluator + eval_nodes/eval_node
│   ├── rule.rs       (136)  eval_rule + combine_selectors
│   ├── value.rs      (524)  eval_value + binop + 算术运算
│   ├── control_flow.rs(149) eval_if/for/each/while
│   ├── mixin.rs      (192)  eval_include + call_function + call_user_function
│   ├── extend.rs     (77)   apply_extends
│   ├── module.rs     (241)  resolve_file + load_module（支持 load_paths）
│   ├── color.rs      (604)  颜色转换 + builtin 颜色函数
│   ├── builtin.rs    (298)  call_builtin 分派入口
│   └── builtin/
│       ├── color.rs    (253)  invert/grayscale/hwb/complement/adjust-hue/...
│       ├── list.rs     (259)  length/nth/append/join/index/zip/...
│       ├── map.rs      (301)  map-get/keys/values/has-key/merge/remove/set + 嵌套辅助
│       ├── string.rs   (281)  str-length/slice/index/insert/split/unquote/quote/...
│       └── selector.rs  (98)  selector-append/nest/is-super/parse/...
└── stage/                  管线阶段类型
```

## Git 规范

- 分支：`main`（发布）/ `v2-rewrite-from-scratch`（开发）/ `perf/optimization`（优化）
- 推送：`git push gitee main`
- Commit 格式：`feat: 描述 — 总计 N/M`
- 不主动 commit/push 除非用户要求

## 当前状态

- sass-spec: 1843/5069 (36%)
- 28 compile + 10 stage + 8 ast + 5 diff 测试全通过（物理隔离，全部在 tests/ 目录）
- Bootstrap 5.3.8：`bootstrap.scss` 全量编译通过 ✅
- Element Plus：121/121 (100%) 全量通过 ✅
- 最大源文件 623 行（`parse/expr.rs`），3 个文件超 500 行（expr.rs/color.rs/value.rs 待拆分）
- 调试工具链：CSS diff + 最小化 + 值快照 events
- 已删除 libsass/non_conformant 目录
- 已实现：load path 支持 + @return 控制流传播 + map 嵌套操作 + str-split + compatible + 字符串转义 + @import 环境继承 (load_import) + and/or 短路求值 (is_truthy) + @while/@each 环境传播 + 插值拼接 (#{...}ident) + bind_params spread Map → 关键字参数 + url() 分流（字符串参数走正常解析，裸 URL 走 raw）+ CSS 函数名大小写不敏感 (to_lowercase) + CSS transform/filter 白名单 + zip 非列表参数视为单元素列表 + MAX_DEPTH=100000（内存爆炸兜底）+ 命名颜色反向查找 (reverse_lookup_named_color) + invert/grayscale CSS 透传 + call 内建函数支持用户函数
- 剩余瓶颈：oklch/oklab 未实现；@extend 选择器引擎需结构化类型；sass-spec 回归需排查（2003→1843）
- 版本：v0.3.0
- OpenSpec change: v2-rewrite-from-scratch (已归档到 main)

## 验证清单

修复后必须运行以下全部验证，确认无回归：

```bash
# 1. compile 测试（28 个，秒级）
cargo test --test compile_test

# 2. stage 测试（10 个，秒级）
cargo test --test stage_test

# 3. ast 测试（8 个，秒级）
cargo test --test ast_test

# 4. diff 测试（5 个，秒级）
cargo test --test common_test

# 5. Bootstrap 全量编译（15 个测试，秒级）
cargo test --test bs_spec -- --nocapture

# 6. Element Plus 全量编译（121 个文件，约 25 秒）
cargo test --test ep_full -- --nocapture
```

**全部通过标准**：compile 28/28 + stage 10/10 + ast 8/8 + diff 5/5 + BS 15/15 + EP 121/121
