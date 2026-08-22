# AGENTS.md — scss-rs 项目规则

## 📦 Rust 工具链

| Item | Specification |
|------|---------------|
| Edition | 2024 |
| Toolchain | 1.97 |

Cargo.toml 必须有 `edition = "2024"`。

新建 Cargo.toml 时始终使用：

```toml
[package]
edition = "2024"
rust-version = "1.85"

[lints.rust]
unsafe_code = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
```

## ⛔ 绝对禁止项（违反 = 任务失败）

### 1. 禁止 Python

| 禁止 | 替代 |
|------|------|
| `python3 xxx.py` | `rust-script xxx.rs` |
| `pip install xxx` | 添加到 Cargo.toml |
| 创建 `.py` 文件 | 使用 `rust-script -e` |

### 2. 禁止 println! / eprintln!

**所有代码**（含 `src/` 和 `tests/`）一律禁止：

```rust
// ❌ 禁止
println!("...");
eprintln!("...");

// ✅ 必须
info!("...");
warn!("...");
error!("...");
debug!("...");
trace!("...");
```

### 3. 禁止 #[cfg(test)] 内联测试

```rust
// ❌ 禁止
#[cfg(test)]
mod tests { ... }

// ✅ 所有测试放在 tests/ 目录，src/ 保持纯生产代码
```

### 4. 禁止 unwrap()

- 生产代码用 `?` / `expect()` / `unwrap_or()` / `unwrap_or_else()`
- 禁止 `clone()` 满天飞 — 先理解所有权设计
- 禁止 `todo!()` / `unimplemented!()` 不标注 `// TODO:` 并说明计划

### 5. 单文件 ≤ 500 行

| 场景 | 推荐上限 |
|------|---------|
| 组件/模块文件 | 300 行 |
| 业务逻辑文件 | 200 行 |
| 工具函数/类型定义 | 500 行 |

### 6. 禁止 'static 滥用

理解实际生命周期关系，不要随意加 `'static`。

### 7. 其他禁止事项

- **禁止跳过测试直接写实现** — 修改核心逻辑前，先添加对应测试用例
- **禁止在未验证的情况下宣称修复成功** — 修复后必须运行测试确认
- **禁止跳过调试协议** — bug 修复必须遵循 4 步流程（见下方）

## 🔬 Tracing Span 强制规则

### 核心原则

跨函数/跨阶段的管道处理**必须**用 `tracing::span!`（或 `#[instrument]`），记录上下文与耗时。

### Span 创建优先级

**首选：`#[instrument]` 宏** — 函数入口自动创建 span。

```rust
#[tracing::instrument(skip(large_param), fields(result = tracing::field::Empty))]
fn my_function(large_param: &BigType, input: &str) -> Result<...> { ... }

#[tracing::instrument(ret)]
fn my_function(input: &str) -> i32 { 42 }

#[tracing::instrument(err)]
fn my_function(input: &str) -> Result<(), std::io::Error> { Ok(()) }
```

**备选：`.entered()` — 条件分支/内联代码块**

```rust
let _span = info_span!("parse_expr", expr = ?input).entered();
```

### 必需业务字段

| 字段 | 用途 |
|------|------|
| `stage` | 管道阶段（lexer/parser/eval/serialize） |
| `module` | 功能模块 |
| `id` | 节点/语句标识 |
| `expr` | 表达式内容 |
| `value` | 求值结果 |
| `elapsed_ms` | 耗时（毫秒） |
| `error` | 错误消息 |
| `file` | 源文件路径 |

## 🔬 调试协议（4 步强制流程）

### Step 1: SPAN 插桩
### Step 2: TRACE 采集（`RUST_LOG=trace cargo test`）
### Step 3: 根因定位（必须引用 span + 字段值）
### Step 4: 修复验证

## 🏗 项目核心

scss-rs 是纯 Rust SCSS 编译器。架构：

```
Source ──► Lexed ──► Parsed ──► Evaluated ──► Serialized
(lex/)   (parse/)  (eval/)     (css/)
```

### 管线类型状态机

每个阶段是一个 struct，阶段间通过 `TryFrom` 转换：

```rust
impl TryFrom<Source> for Lexed { ... }
impl TryFrom<Lexed> for Parsed { ... }
impl TryFrom<Parsed> for Evaluated { ... }
```

### Env 设计（move 语义）

- `enter_scope(&self) -> Env`：创建子作用域
- `exit_scope(self, child: &Env) -> Env`：从子作用域提取传播字段
- Builder 方法：`with_xxx(xxx) -> Self`、`define_xxx(name, val) -> Self`
- 只读方法：`get_xxx() -> &T`
- `eval_nodes` 返回 `(Vec<CssNode>, Env)` — 允许模块系统获取最终环境

### 模块系统

- `file_resolver.rs` — 文件路径解析（partial/extension/index/import-only 四种冲突检测）
- `module.rs` — @use 文件加载 + 模块缓存、@forward show/hide/prefix 过滤、@import 内联
- `module_helpers.rs` — bind_exports + merge_module_cache

### @extend 后处理

- `extend.rs` — 选择器匹配 + 替换 + bogus extend 跳过 + !optional 抑制
- `plain_css.rs` — CSS @import 提升到顶部（hoist_css_imports）

### 内建函数 dispatch（const 静态表）

- `BUILTIN_TABLE: &[BuiltinEntry]` — 编译期 const 注册
- 无 proc-macro，无运行时反射
- 三个函数从同一张表生成：`module_builtin_name`、`is_known_builtin`、`dispatch_builtin`
- 已实现 7 个模块：math（trig/log/pow/sqrt/clamp/hypot 等）、string（split/quote/unquote/upper/lower/index/insert/slice）、map（get/merge/remove/keys/values/has-key/deep-merge/deep-remove）、list（length/nth/set-nth/join/append/zip/index/separator/slash）、color（mix/adjust/change/scale 骨架）、meta（call/type-of/inspect/feature-exists/function-exists/get-function/get-mixin 等）、selector（nest/append/parse/is-superselector）

### 值系统

- `Value` 枚举支持 AST 级别延迟求值（BinOp/UnaryOp/Call/Interp/Calc/Paren）
- `Color` 结构含 RGB + Alpha + ColorFormat（Auto/Rgb/Hsl/Hwb）
- `equals` 支持 Number/String/Ident/Bool/Null/Color/List/Map 比较

## ✅ 验证清单

```bash
cargo test --test compile_test    # 19 个
cargo test --test lex_test        # 29 个
cargo test --test bs_spec          # 15 个 Bootstrap
cargo test --test ep_full          # Element Plus 全量
cargo test --test sass_spec_full   # sass-spec 全量统计
```

**当前基线**：19/19 + 29/29 + 15/15 + 1235/5362 (23%) + ep_full 10/121

## 🔄 Git 规范

| 规则 | 说明 |
|------|------|
| 推送方式 | SSH：`git push origin scss-rs` |
| Commit 格式 | `feat: 描述` |
| 只提交不推送 | commit 后必须等用户确认再推送 |

## OpenSpec

- `openspec/config.yaml` — 项目上下文（每次 openspec 命令自动加载）
- 已归档变更：`align-sasspile`（2026-08-22 归档）— 自底向上逐层补全（8 层 59 个任务，sass-spec 5%→23%）

## 🔍 CodeGraph

```bash
codegraph sync          # 同步索引
codegraph callers <fn>  # 调用链
codegraph impact <fn>   # 影响分析
codegraph explore "query" # 自然语言探索
```
