---
name: tracing-debug
description: |
  CRITICAL: Use for ALL debugging and error diagnosis in sasspile.
  MANDATORY before proposing any fix. Must trace the error chain using tracing spans.
  Triggers: sass-spec failure, test failure, compile error, wrong output, bug, 修复, 调试, diagnose, debug, trace, 追踪
globs: ["**/*.rs", "**/Cargo.toml"]
---

# Tracing Debug — sasspile 错误追踪技能

> **强制规则**：修复任何 sasspile bug 前，必须先用 tracing 追踪完整错误链路。

## 原则

1. **先追踪，再修复** — 不允许未追踪错误链路就提出修复方案
2. **Span + Event** — Span 追踪结构（WHERE），Event 追踪值（WHAT）
3. **Per-target 过滤** — 用 `RUST_LOG="target=level"` 精准过滤关注的模块
4. **最小化假设** — 每次只改一个变量，用 tracing 验证效果
5. **证据驱动** — 修复后必须用 tracing 确认错误消失

## Tracing 架构

sasspile 已在关键路径部署 tracing span：

### Span 层级树

```
eval_nodes                    # 节点列表求值 (trace, fields: depth, n)
├── eval_node_item            # try_fold 每个节点 (debug, fields: node discriminant)
│   └── eval_node             # 单节点求值 (trace, fields: depth)
│       ├── eval_rule         # 规则求值 (info, fields: selector)
│       │   └── eval_nodes    # 递归求值子节点
│       ├── eval_for          # @for 循环 (info, fields: var, inclusive)
│       ├── eval_each         # @each 循环 (info, fields: n_vars)
│       ├── eval_include      # @include mixin (info, fields: name, n_args)
│       └── eval_value        # 值表达式 (trace, fields: depth)
│           ├── call_function     # 函数调用入口 (info, fields: name, n_args)
│           │   ├── call_builtin      # 内建函数 (info, fields: name, n_args)
│           │   ├── call_module_function # 模块函数 (info, fields: name)
│           │   └── call_user_function   # 用户函数 (info, fields: n_params, n_args)
│           └── eval_interp_str  # 插值求值
├── load_module               # 文件模块加载 (info, fields: path, depth, n_config)
│   └── resolve_file          # 文件路径解析 (debug, fields: url)
└── apply_extends             # @extend 后处理
```

### 错误记录点

`eval_nodes` 的 `try_fold` 中，每个节点的错误都会被记录：

```rust
let (mut out, new_env) = Self::eval_node(node, &env).map_err(|e| {
    tracing::error!(error = %e, node_type = ?std::mem::discriminant(node), "eval_node failed");
    e
})?;
```

## 调试工作流

### 步骤 1：复现错误

```bash
# 用 cf_diag 诊断特定子目录（集成 CSS 逐行 diff）
cargo test --test cf_diag diag_<subdir> -- --nocapture 2>&1 | grep -E "FAIL|ERROR" -A2 | head -20

# CSS diff 详情模式——看每行差异
RUST_LOG="cssdiff=debug" cargo test --test cf_diag diag_<subdir> -- --nocapture

# 用最小化工具自动最小化失败用例
RUST_LOG="minimize=info" cargo test --test minimize minimize_color_error -- --nocapture

# 或用 lib 测试中的 test_debug_bs_close 复现
# 修改 src/lib.rs 中的 test_debug_bs_close input
RUST_LOG=error cargo test --lib test_debug_bs_close -- --nocapture
```

### 步骤 2：追踪错误链路

```bash
# Level 1: 只看错误（最快）
RUST_LOG=error cargo test --lib test_debug_bs_close -- --nocapture

# Level 2: 看函数调用链（推荐）
RUST_LOG=info cargo test --lib test_debug_bs_close -- --nocapture

# Level 3: 看完整 span 嵌套（最详细）
RUST_LOG=debug cargo test --lib test_debug_bs_close -- --nocapture

# Level 4: 看所有 trace（含值表达式）
RUST_LOG=trace cargo test --lib test_debug_bs_close -- --nocapture

# Per-target 过滤——只看颜色转换值快照
RUST_LOG="sasspile::color=trace" cargo test --lib -- --nocapture

# Per-target 过滤——只看 @extend 匹配
RUST_LOG="sasspile::extend=debug" cargo test --lib -- --nocapture

# Per-target 过滤——只看 BinOp 运算值
RUST_LOG="sasspile::binop=trace" cargo test --lib -- --nocapture

# 组合多个 target
RUST_LOG="sasspile::color=debug,sasspile::extend=info" cargo test --lib -- --nocapture

# OTel 追踪（输出 OpenTelemetry span，含 TraceId/SpanId/busy_ns 精确耗时）
RUST_LOG=info cargo test --features otel --test compile_test <name> -- --nocapture
RUST_LOG=info cargo test --features otel --test sass_spec_full test_sass_spec_full_stats -- --nocapture
```

### 步骤 3：分析 span 链路

错误日志中的 span 嵌套展示了完整调用路径：

```
ERROR eval_nodes:eval_node_item:eval_node:eval_rule:eval_nodes:eval_node_item: 
  eval_node failed 
  error=未定义函数: undefined-fn 
  node_type=Discriminant(1) depth=0 n=1 
  selector="a" depth=0
```

解读：
- `eval_nodes` → 顶层节点列表
- `eval_node_item` → try_fold 中的某个节点
- `eval_node` → 单节点求值
- `eval_rule` → 进入规则 `a { ... }`
- `eval_nodes` → 规则体内的节点列表
- `eval_node_item` → 规则体内的某个节点（声明）
- `error=未定义函数` → 根因：函数未定义

### 步骤 4：定位根因

根据 span 链路中的字段信息：

| Span 字段 | 含义 | 用途 |
|-----------|------|------|
| `selector` | CSS 选择器文本 | 定位是哪个规则出错 |
| `name` | 函数/mixin 名 | 定位是哪个函数调用出错 |
| `n_args` | 参数数量 | 检查参数数量是否匹配 |
| `path` | 文件路径 | 定位是哪个文件加载出错 |
| `url` | 模块 URL | 定位 @use/@import 路径 |
| `depth` | 递归深度 | 检测无限递归 |
| `var` | 循环变量名 | @for 循环上下文 |
| `n_config` | with() 配置数 | 模块配置上下文 |

### 步骤 5：修复并验证

```bash
# 修复后验证——错误消失
RUST_LOG=error cargo test --lib test_debug_bs_close -- --nocapture

# 全量统计验证
RUST_LOG=info cargo test --test sass_spec_full test_sass_spec_full_stats -- --nocapture 2>&1 | grep "全量"

# OTel 追踪验证（输出 span 到 stdout）
RUST_LOG=info cargo test --features otel --test sass_spec_full test_sass_spec_full_stats -- --nocapture

# 确保无回归
cargo test --lib 2>&1 | tail -3
```

## 添加新 Span

修复新问题时，如果发现关键路径缺少 span，先添加再调试：

```rust
// 手动 span（适用于条件分支）
let span = tracing::info_span!("my_function", key_field = value);
let _enter = span.enter();

// 属性 span（适用于函数入口）
#[instrument(skip(large_param), fields(context_field = value))]
fn my_function(large_param: &BigType, context_field: &str) -> Result<...> {
    ...
}

// 错误记录
result.map_err(|e| {
    tracing::error!(error = %e, "my_function failed");
    e
})?;
```

### Span 级别规范

| 级别 | 使用场景 | 示例 |
|------|---------|------|
| `error` | 错误发生时 | `eval_node failed` |
| `warn` | 可恢复的异常 | `recursion limit exceeded` |
| `info` | 关键路径入口 | `eval_rule`, `call_function`, `load_module` |
| `debug` | 次要路径 | `resolve_file`, `eval_node_item` |
| `trace` | 高频调用 | `eval_value`, `eval_node` |

## 常见错误模式速查

| 错误信息 | 根因 | 修复方向 |
|----------|------|---------|
| `未定义函数: xxx` | 函数未注册或映射缺失 | 检查 `call_module_function` 映射表 |
| `未定义变量: $xxx` | 变量作用域问题 | 检查 `env.lookup` 和 `@use` 命名空间 |
| `求值错误: xxx 需要 N 个参数` | 参数数量不匹配 | 检查 `eval/builtin.rs` 的 match 分支 |
| `语法错误: 期望 X, 实际 Y` | 解析器不支持该语法 | 检查 `parse_value` / `parse_args` |
| `词法错误: 无效字符` | 词法分析器不支持该字符 | 检查 `Lexer` 字符处理 |
| `模块加载: 无法读取` | 文件路径解析失败 | 检查 `resolve_file` 候选路径 |
| `content_diff` | 输出内容差异 | 用 `RUST_LOG="cssdiff=debug"` 看逐行 diff |
| `missing_output` | 实际输出缺少行 | 检查 CSS 序列化和 @import/@forward |
| `extra_output` | 实际输出多余行 | 检查规则嵌套和选择器组合 |

## Event Targets 值快照

> **Span = 结构（WHERE），Event = 值（WHAT）**

除了 Span 层级树，sasspile 还在关键决策点部署了 Event 值快照：

| Target | Level | 场景 | 示例 |
|--------|-------|------|------|
| `sasspile::color` | trace | 颜色转换函数输入/输出 | `h=120 s=0.5 l=0.5 converting HSL to RGB` |
| `sasspile::color` | debug | 颜色 builtin 函数入口/结果 | `input_r=242 amount=30 darken input` |
| `sasspile::extend` | info | @extend 匹配成功 | `extender=.large target=.btn extend matched` |
| `sasspile::extend` | debug | 选择器替换细节 | `new_selector=.large placeholder replaced` |
| `sasspile::binop` | trace | 二元运算操作数值 + 结果 | `op=Sub left=c right=d binop result` |
| `cssdiff` | info | CSS diff 检测摘要 | `n_expected=3 n_actual=2 diff detected` |
| `cssdiff` | debug | 行级差异详情 | `line=1 expected='red' actual='blue'` |
| `minimize` | info | 最小化轮次摘要 | `round=1 n_nodes=2 new round` |
| `minimize` | debug | 每次移除尝试 | `removed_node=Discriminant(3) trying removal` |

## 调试工具

### CSS Diff 模块 (`tests/common/mod.rs`)

逐行对比期望 vs 实际 CSS 输出，分类统计差异类型：
- `content_diff` — 行内容不同
- `missing_output` — 实际输出缺少行
- `extra_output` — 实际输出多余行

集成在 `cf_diag.rs` 中，失败时自动显示前 3 个差异行。

### sass-spec 最小化工具 (`tests/minimize.rs`)

Delta debugging 算法，自动将失败用例最小化到最小复现代码：

```bash
# 最小化颜色错误用例
RUST_LOG="minimize=info" cargo test --test minimize minimize_color_error -- --nocapture

# 最小化 @extend 错误用例
RUST_LOG="minimize=info" cargo test --test minimize minimize_extend_error -- --nocapture
```

两种 oracle 模式：
- `Error` — 编译仍然报错
- `OutputPreserve` — 输出与原始（错误）输出相同

### Node::to_scss() (`src/parse/ast_impl.rs`)

AST → SCSS 序列化方法，支持最小化工具将修改后的 AST 转回可编译的 SCSS 源码。

## 禁止事项

- **禁止未追踪就修复** — 必须先用 `RUST_LOG` 看错误链路
- **禁止猜测根因** — 用 span 链路定位，而非猜测
- **禁止批量修改** — 每次只改一个变量，用 tracing 验证
- **禁止跳过验证** — 修复后必须确认错误消失且无回归
