# 调试工具链设计：CSS Diff + 最小化 + 值快照 Events

> **日期**: 2026-08-10
> **状态**: 已批准，待实现
> **关联**: sasspile v2-rewrite-from-scratch

## 1. 背景与动机

### 1.1 当前痛点

sass-spec 通过率 2322/10632 (21.9%)，剩余瓶颈集中在：
- **core_functions/color** (5473 失败) — 颜色空间转换逻辑复杂
- **@extend 选择器引擎** — 缺少结构化选择器类型

调试这三个痛点时遇到三个具体困难：
1. **输出对比困难** — `cf_diag` 只比较第一行，看不到具体差异在哪
2. **难以复现** — 失败用例可能几十行 SCSS，不知道哪个节点触发 bug
3. **领域逻辑复杂** — tracing 只有 Span（结构），缺 Event（值），看不到中间值

### 1.2 不采用 OpenTelemetry 的理由

sasspile 是单进程 CLI/库编译器，没有服务边界。OpenTelemetry 的核心价值是分布式追踪，对单进程编译器收益为零。三个痛点都是本地调试问题，需要的是 **diff 工具、最小化工具、值快照 events**，而非分布式 tracing exporter。

### 1.3 设计原则

> **Span = 结构（WHERE），Event = 值（WHAT）**

现有 Span 层级树保留不动。新增 Event 在关键决策点记录值快照。新增两个调试工具（CSS diff、最小化）和 init_tracing 升级。

## 2. 工具 1：共享 CSS Diff 模块

### 2.1 文件

`tests/common/mod.rs` — 被 `cf_diag.rs` 和 `sass_spec_full.rs` 共享。

### 2.2 API

```rust
/// CSS 逐行 diff——返回结构化差异结果。
///
/// 发出 tracing events：
/// - `cssdiff` info: diff 检测摘要
/// - `cssdiff` debug: 每行差异详情
pub fn diff_css(expected: &str, actual: &str) -> DiffResult

pub enum DiffLine {
    Changed { line: usize, expected: String, actual: String },
    ExtraExpected { line: usize, content: String },
    ExtraActual { line: usize, content: String },
}

pub struct DiffResult {
    pub lines: Vec<DiffLine>,
}

impl DiffResult {
    /// 分类错误模式（用于统计）。
    pub fn classify(&self) -> &str
    /// 格式化为终端可读文本。
    pub fn format_terminal(&self) -> String
}
```

### 2.3 Event 设计

| target | level | 场景 |
|--------|-------|------|
| `cssdiff` | `info` | diff 检测（n_expected, n_actual） |
| `cssdiff` | `debug` | 行级差异（line, expected, actual） |

### 2.4 集成点

- `cf_diag.rs` 的 `diag()` 函数：替换只比较第一行的逻辑为 `diff_css()` 调用
- summary 模式（默认）：显示前 3 个差异行
- 通过 `RUST_LOG="cssdiff=debug"` 看完整差异

### 2.5 不引入 `similar` crate

第一版用简单逐行比对。后续如果需要 LCS（最长公共子序列）算法再引入 `similar = "2"`。

## 3. 工具 2：sass-spec 最小化工具

### 3.1 文件

`tests/minimize.rs`

### 3.2 算法：Delta Debugging

```
输入：原始 SCSS（失败）
输出：最小 SCSS（仍然失败）

1. 解析为 AST，取 nodes: Vec<Node>
2. 遍历 nodes，尝试移除每个节点
3. 序列化剩余 AST → SCSS（Node::to_scss()）
4. 编译，检查是否仍然失败（oracle 判定）
5. 如果仍然失败 → 保留移除
6. 如果不再失败 → 恢复
7. 重复直到没有节点可以移除
```

### 3.3 失败 Oracle

```rust
enum FailOracle<'a> {
    /// 错误模式：编译仍然报错
    Error,
    /// 输出保持模式：输出与原始（错误）输出相同
    /// 移除不影响输出的无关节点，保留导致错误的节点
    OutputPreserve { original_output: &'a str },
}

impl FailOracle<'a> {
    fn still_fails(&self, input: &str) -> bool
}
```

### 3.4 前置依赖：Node::to_scss()

在 `src/parse/ast.rs` 中添加 `Node::to_scss(&self, indent: usize) -> String` 方法。
`Value` 已有 `Display` 实现，直接复用。

覆盖所有 Node 变体：Rule, Decl, Variable, Comment, If, For, Each, While, MixinDef, Include, Content, FunctionDef, Return, Use, Forward, Import, Extend, AtRoot, AtRule, Warn, Debug, Error。

### 3.5 Event 设计

| target | level | 场景 |
|--------|-------|------|
| `minimize` | `info` | 最小化开始/完成、每轮摘要 |
| `minimize` | `info` | 移除成功（still_fails=true） |
| `minimize` | `debug` | 每次移除尝试 |
| `minimize` | `debug` | 恢复移除（still_fails=false） |
| `minimize` | `warn` | OutputPreserve 模式下编译失败 |

### 3.6 限制

- 第一版只处理单文件用例（多文件 HRX 留到 V2）
- OutputPreserve 模式对 diff 失败用例有效，但不是所有 diff 都能最小化
- 后续可加值简化（不只是移除节点，还尝试简化 Value 表达式）

## 4. 工具 3：color/selector 值快照 Events

### 4.1 颜色转换函数

在 `hsl_to_rgb`、`hwb_to_rgb`、`rgb_to_hsl` 中添加 trace 级 events：
- 转换前：输入值（h, s, l 或 r, g, b）
- 转换后：输出值

### 4.2 颜色 builtin 函数

在 `builtin_darken`、`builtin_lighten`、`builtin_mix`、`builtin_rgba` 中添加 debug 级 events：
- 函数入口：输入颜色值 + 参数
- 函数出口：结果颜色值

### 4.3 @extend 选择器匹配

在 `apply_extends` 中添加：
- Span：`apply_extends`（info_span, n_extends）
- Event：规则处理（debug, selector）
- Event：匹配成功（info, extender, target, selector）
- Event：占位符替换（debug, new_selector）
- Event：继承者添加（debug, final_selector）

### 4.4 BinOp 值快照

在 BinOp 求值中添加 trace 级 events：
- 操作数求值后（op, left, right）
- 结果计算后（op, result）

### 4.5 Event 级别规范

| target | level | 场景 |
|--------|-------|------|
| `sasspile::color` | `trace` | 转换函数输入/输出（高频） |
| `sasspile::color` | `debug` | builtin 函数入口/结果 |
| `sasspile::extend` | `info` | 匹配成功的 extend |
| `sasspile::extend` | `debug` | 选择器替换细节 |
| `sasspile::binop` | `trace` | 运算符操作数值 |
| `cssdiff` | `info` | diff 检测 |
| `cssdiff` | `debug` | 行级差异 |
| `minimize` | `info` | 最小化轮次摘要 |
| `minimize` | `debug` | 每次移除尝试 |

## 5. init_tracing 升级

### 5.1 改动

将 `.with_target(false)` 改为 `.with_target(true)`，新增 `.with_level(true)` 和 `.with_ansi(true)`。

```rust
pub fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn"));
    let _ = fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_ansi(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .try_init();
}
```

### 5.2 不需要新依赖

`EnvFilter` 已支持 per-target 语法，`Cargo.toml` 不变。

### 5.3 兼容性

- 65 个 lib 测试不受影响（只改变终端输出格式）
- 现有 `RUST_LOG=info` 命令正常工作
- 新增 target 过滤是纯增量

## 6. 使用示例

```bash
# 只看颜色转换 trace
RUST_LOG="sasspile::color=trace" cargo test --lib test_debug_bs_close -- --nocapture

# 只看 @extend 匹配
RUST_LOG="sasspile::extend=debug" cargo test --lib test_compile_extend -- --nocapture

# CSS diff 详情
RUST_LOG="cssdiff=debug" cargo test --test cf_diag diag_color -- --nocapture

# 最小化摘要
RUST_LOG="minimize=info" cargo test --test minimize minimize_color_error -- --nocapture

# 组合多个 target
RUST_LOG="sasspile::color=debug,sasspile::extend=info,cssdiff=info" \
    cargo test --test cf_diag diag_color -- --nocapture
```

## 7. 文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/lib.rs` | 修改 | `init_tracing` 升级 |
| `src/parse/ast.rs` | 修改 | 新增 `Node::to_scss()` |
| `src/eval/mod.rs` | 修改 | color/extend/binop events |
| `tests/common/mod.rs` | 新建 | 共享 CSS diff 模块 |
| `tests/cf_diag.rs` | 修改 | 集成 diff_css |
| `tests/minimize.rs` | 新建 | delta debugging 最小化工具 |
| `Cargo.toml` | 不变 | 无新依赖 |

## 8. 不在范围内

- 不引入 OpenTelemetry / Jaeger / Zipkin
- 不引入 `similar` crate（第一版逐行 diff）
- 不重构 `call_builtin` 为模块化分派（长期方向，另开设计）
- 不实现新的颜色空间转换（LAB/OKLCH 等）
- 不实现结构化选择器类型
- 多文件 HRX 最小化（留到 V2）
