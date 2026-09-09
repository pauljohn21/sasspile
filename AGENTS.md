> ⛔ **禁止参照 dart-sass**：dart-sass 依赖 GC（垃圾回收），其嵌套结构依赖 GC 保。sasspile 是纯 Rust 项目，无 GC，所有权语义完全不同。任何实现必须基于 Rust 所有权模型和 sass-spec 规范，不得参照 dart-sass 的实现。

# AGENTS.md — sasspile 项目规则

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

超过 **500 行**的文件必须先拆分再编写（源码和测试分别计算）。

### 6. 禁止 'static 滥用

理解实际生命周期关系，不要随意加 `'static`。

### 7. 函数式 Rust 强制规则（第一公民）

函数式风格是 sasspile 的核心设计哲学。以下规则**不可违反**。

#### 7.1 所有权：move 优先，禁止 clone 满天飞

| 禁止 | 替代 | 说明 |
|------|------|------|
| `env.clone()` | `env` move 进函数，返回 `(T, Env)` | 零拷贝传递 |
| `&mut Env` 参数 | `Env`（move）→ `self -> Self` 链式 | 不可变借用 + 返回新值 |
| `Rc<RefCell<T>>` | 按值传递 + 返回新值 | 避免 interior mutability |
| `&self` + clone 返回 | `self` 消费 + `into_xxx()` | 类型状态机模式 |

```rust
// ❌ 禁止：clone + 修改
fn eval_nodes(nodes: &[Node], env: &Env) -> Vec<CssNode> {
    let mut env = env.clone();
    env.bind("x", Value::Number(1.0, None));
    // ...
}

// ✅ 正确：move + 返回新状态
fn eval_nodes(nodes: &[Node], env: Env) -> (Vec<CssNode>, Env) {
    let env = env.bind("x", Value::Number(1.0, None));
    // ...
}
```

#### 7.2 迭代器：禁止显式 for 循环处理集合变换

| 禁止 | 替代 | 场景 |
|------|------|------|
| `for x in vec { result.push(f(x)) }` | `vec.into_iter().map(f).collect()` | map 变换 |
| `for x in &vec { if pred(x) { ... } }` | `vec.into_iter().filter(pred)...` | filter 筛选 |
| `for x in vec { match ... { Ok(v) => acc.push(v), Err(e) => return e } }` | `vec.into_iter().try_fold(acc, ...)` | 错误传播累积 |
| `for x in vec { if pred(x) { left.push(x) } else { right.push(x) } }` | `vec.into_iter().partition(pred)` | 分流 |
| 可变 `Vec` + push + extend | `flat_map` / `flatten` | 展平嵌套 |
| `for (i, x) in vec.iter().enumerate()` | `vec.into_iter().enumerate()` | 带索引 |

```rust
// ❌ 禁止：命令式可变累积
let mut result = Vec::new();
for node in nodes {
    if node.is_css() {
        result.push(transform(node));
    }
}

// ✅ 正确：函数式迭代器链
nodes.into_iter()
    .filter(|n| n.is_css())
    .map(transform)
    .collect::<Vec<_>>()

// ✅ 更好：带错误传播
nodes.into_iter()
    .try_fold(Vec::new(), |mut acc, node| {
        acc.push(transform(&node)?);
        Ok::<_, SassError>(acc)
    })
```

#### 7.3 模式匹配：禁止 if-else 链处理枚举

```rust
// ❌ 禁止：if-else 链
if token == "{" { ... }
else if token == "}" { ... }
else if token == ";" { ... }
else { ... }

// ✅ 正确：match
match token {
    "{" => ...,
    "}" => ...,
    ";" => ...,
    _ => ...,
}
```

#### 7.4 副作用：禁止 &mut 参数

| 禁止 | 替代 | 说明 |
|------|------|------|
| `fn f(buf: &mut String, x: &str)` | `fn f(x: &str) -> String` | 返回新值 |
| `fn f(items: &mut Vec<T>, n: usize)` | `fn f(items: Vec<T>, n: usize) -> Vec<T>` | 消费 + 返回 |
| `fn f(env: &mut Env, node: &Node)` | `fn f(env: Env, node: &Node) -> (Vec<CssNode>, Env)` | move 语义 |

#### 7.5 函数签名的强制模式

| 场景 | 签名模板 | 说明 |
|------|----------|------|
| 数据变换 | `fn transform(input: Input) -> Output` | 消费输入，返回新值 |
| 带状态变换 | `fn step(state: State, input: Input) -> (Output, State)` | move 语义，返回新状态 |
| 管线阶段 | `fn next_stage(self) -> Result<NextStage>` | `self` 消费，类型状态机 |
| 链式构建 | `fn with_x(mut self, x: X) -> Self` | builder 模式 |
| 只读查询 | `fn query(&self, key: &str) -> Option<&Value>` | 纯函数，不可变借用 |

#### 7.6 错误处理：禁止 match Err 分支

```rust
// ❌ 禁止：显式 match Err
let result = match parse(tokens) {
    Ok(ast) => ast,
    Err(e) => return Err(e),
};

// ✅ 正确：? 传播
let ast = parse(tokens)?;
```

### 8. 其他禁止事项

- **禁止跳过测试直接写实现** — 修改核心逻辑前，先添加对应测试用例
- **禁止在未验证的情况下宣称修复成功** — 修复后必须运行测试确认
- **禁止跳过调试协议** — bug 修复必须遵循 4 步流程（见下方）

## 🔬 Tracing Span 强制规则

### 核心原则

跨函数/跨阶段的管道处理**必须**用 `tracing::span!`（或 `#[instrument]`），记录上下文与耗时。**禁止仅用 event! 单一日志**。

### Span 创建优先级

**默认首选：`#[instrument]` 宏** — 函数入口自动创建 span，参数自动记录为字段。

```rust
// ✅ 首选：函数入口用 #[instrument]
#[tracing::instrument(skip(large_param), fields(result = tracing::field::Empty))]
fn my_function(large_param: &BigType, input: &str) -> Result<...> {
    let result = do_work(large_param, input)?;
    tracing::Span::current().record("result", &result);
    Ok(result)
}

// ✅ 返回值自动记录
#[tracing::instrument(ret)]
fn my_function(input: &str) -> i32 { 42 }

// ✅ 错误自动记录
#[tracing::instrument(err)]
fn my_function(input: &str) -> Result<(), std::io::Error> { Ok(()) }

// ✅ async 函数（自动处理 span 跨 await）
#[tracing::instrument]
async fn my_async_fn() { /* ... */ }
```

**备选 1: `.entered()` — 条件分支/内联代码块**

```rust
let _span = info_span!("parse_expr", expr = ?input, pos = self.pos).entered();
// ... logic ...
// _span drop 时自动退出
```

**备选 2: `enter()` + `field::Empty` — 需要延迟记录返回值**

```rust
let span = info_span!("eval", result = tracing::field::Empty);
let _enter = span.enter();
// ... 计算 result ...
span.record("result", &result);  // 传原始值，不用 format!
```

### Field Value Recording（官方语法）

| Sigil | Example | Trait Used | 说明 |
|-------|---------|------------|------|
| `?` | `field = ?value` | `fmt::Debug` | 调试格式化 |
| `%` | `field = %value` | `fmt::Display` | 显示格式化 |
| (none) | `field = value` | `tracing::Value` | 需实现 Value trait |
| shorthand | `field` | 同 `field = field` | 局部变量简写 |

```rust
// ✅ 正确
info_span!("eval", expr = ?ast, selector = %s);

// ❌ 错误：手动 format! 传给 record
span.record("result", &format!("{:?}", result));
```

### `#[instrument]` 选项速查

| 选项 | 说明 | 示例 |
|------|------|------|
| `skip(a, b)` | 不记录指定参数 | `#[instrument(skip(self, large))]` |
| `skip_all` | 跳过所有参数 | `#[instrument(skip_all)]` |
| `fields(k = v)` | 添加额外字段 | `#[instrument(fields(next = i + 1))]` |
| `level = "trace"` | 设置级别 | `#[instrument(level = "debug")]` |
| `name = "x"` | 覆盖 span 名 | `#[instrument(name = "my_span")]` |
| `ret` | 记录返回值 | `#[instrument(ret)]` |
| `ret(Display)` | 用 Display 记录返回值 | `#[instrument(ret(Display))]` |
| `err` | 记录 Err 返回值 | `#[instrument(err)]` |

**注意**：`fields` 中定义与参数同名的字段会隐式 skip 该参数。

### 必需业务字段

| 字段 | 用途 |
|------|------|
| `stage` | 管道阶段（lexer/parser/eval/serialize/compile） |
| `module` | 功能模块（import/use/include/extend/for/each/if） |
| `id` | 节点/语句标识 |
| `expr` | 表达式内容 |
| `value` | 求值结果 |
| `token` | 当前 token |
| `node` | AST 节点类型 |
| `elapsed_ms` | 耗时（毫秒） |
| `error` | 错误消息（用 `%` Display sigil） |
| `file` | 源文件路径 |

### 禁止模式

| 禁止 | 原因 |
|------|------|
| 仅用 `event!` 无 `span!` | 无上下文边界 |
| Span 无业务字段 | trace 不可读 |
| `span.record("x", &format!(...))` | 应传原始值或用 `?`/`%` sigil |
| `Span::enter` 跨 await 点 | async 代码 trace 错乱 |

## 🔬 OpenTelemetry (OTel) 追踪架构

### 架构概要

sasspile 通过 `otel` feature 提供可选的 OpenTelemetry 追踪，使用 stdout exporter 输出 span（无需 gRPC/tokio）：

```toml
[features]
default = ["tracing"]
tracing = ["dep:tracing", "dep:tracing-subscriber"]
otel = ["dep:opentelemetry", "dep:opentelemetry_sdk", "dep:opentelemetry-stdout", "dep:tracing-opentelemetry"]
```

### 双模式初始化

| 函数 | feature | 输出 |
|------|---------|------|
| `init_tracing()` | `tracing` | tracing fmt 日志（`RUST_LOG` 控制） |
| `init_tracing_otel()` | `otel` | tracing fmt + OTel stdout span（`RUST_LOG` + `OTEL_SERVICE_NAME`） |

`init_tracing_otel()` 在 `otel` feature 未启用时回退到 `init_tracing()`，无需条件编译。

### OTel Span 输出内容

每个 span 包含：
- **TraceId / SpanId / ParentSpanId** — 完整调用链层级
- **busy_ns / idle_ns** — 精确耗时（纳秒级）
- **业务字段** — `stage`、`module`、`dir`、`hrx`、`pass`、`fail`、`pct` 等
- **Event** — span 内事件（如 `hrx parsed`、`sass-spec 目录`）

### 使用方式

```bash
# 编译器 CLI + OTel
RUST_LOG=info cargo run --features otel -- input.scss

# 测试 + OTel
RUST_LOG=info cargo test --features otel --test sass_spec_full -- --nocapture

# 自定义服务名
OTEL_SERVICE_NAME=sasspile-debug RUST_LOG=debug cargo test --features otel --test compile_test -- --nocapture
```

## 🔬 调试协议（4 步强制流程）

> **核心原则**：禁止凭直觉猜测根因。所有 bug 修复必须基于 tracing trace 证据链。

### Step 1: SPAN 插桩

在疑似路径每个入口/出口加 span：
- **首选 `#[instrument]` 宏** — 函数入口自动创建 span
- 条件分支/闭包用手动 `info_span!`/`debug_span!` + `.entered()`
- 必须携带业务字段（用 `?` Debug / `%` Display sigil）
- 延迟记录的返回值用 `field::Empty` 声明，后续 `.record()` 记录
- **插桩完成前不修改逻辑代码**

### Step 2: TRACE 采集

```bash
# 基础 tracing
RUST_LOG=trace cargo test test_name 2>&1 | tee /tmp/trace.log

# OTel 追踪（输出 OpenTelemetry span 到 stdout，含 TraceId/SpanId/耗时）
RUST_LOG=trace cargo test --features otel test_name -- --nocapture 2>&1 | tee /tmp/trace.log
```

保留 trace 输出作为**证据**。OTel 模式额外提供 `busy_ns`/`idle_ns` 精确耗时和 span 层级 `ParentSpanId`。

### Step 3: 根因定位（必须引用 span + 字段值）

```
Evidence collected:
- span: parse_expr[expr="$i == 1"] → returned Number(1)  ← should be Bool
- span: eval_condition[cond=Number(1)] → missing implicit bool conversion
Root cause: parse_expr doesn't convert Number(1) to true semantically
```

### Step 4: 修复验证

- 修复后重新运行测试，确认错误消失或推进
- **移除临时 debug span**，或降级为 `trace!`/`debug!`
- 保留生产级 span（管道阶段入口、公开 API）

### 简化场景

| 场景 | 处理 |
|------|------|
| 简单拼写/语法错误 | 跳过插桩，注明 "可见错误，无需 tracing" |
| 初始代码探索 | 轻量 `debug_span!` 可接受 |

## 项目核心

sasspile 是纯 Rust 函数式 SCSS 编译器。架构：

```
Source → Lexer → Parser → Evaluator → Serializer → CSS
(lex/)   (parse/)  (eval/)     (css/)
```

### 函数式管线（Reactor 类型状态机 + move 语义）

入口 `lib.rs` 全部链式调用，数据通过 move 语义流过管线：

```rust
// 字符串编译
Reactor::new(input)
    .lex()?
    .parse()?
    .evaluate()?
    .serialize(style)
    .finish()?

// 文件编译
Reactor::from_file(path)?
    .with_load_paths(load_paths)
    .lex()?
    .parse()?
    .evaluate()?
    .serialize(style)
    .finish()?
```

### Reactor 类型状态机

`Reactor<S>` 通过泛型参数 `S` 编码管线阶段，保证编译顺序不可颠倒。

```text
Reactor<StateRaw>.lex()       → Result<Reactor<StateLexed>>
Reactor<StateLexed>.parse()   → Result<Reactor<StateParsed>>
Reactor<StateParsed>.evaluate() → Result<Reactor<StateEvaluated>>
Reactor<StateEvaluated>.serialize() → Reactor<StateSerialized>
Reactor<StateSerialized>.finish()  → Result<String>
```

**Reactor 内部字段**:
- `text: String` — 原始源码
- `base_path: Option<PathBuf>` — 源文件路径
- `load_paths: Vec<PathBuf>` — 模块搜索路径
- `tokens: Option<Vec<Token>>` — 词法产物（lex 后填充）
- `ast: Option<Ast>` — 语法产物（parse 后填充）
- `serialized: Option<String>` — CSS 字符串（serialize 后填充）
- `env: Option<Env>` — 求值环境（evaluate 后填充）

每个管线方法直接调用底层组件（`Lexer` / `Parser` / `Evaluator::evaluate_with_env` / `Serializer`），不再通过中间 stage 类型委托。

### Env 设计（move 语义 + Scope Chain 零 clone）

- `Env` 持有 `current: Rc<Scope>`（当前活跃作用域），通过 parent 链管理嵌套作用域
- `Scope` 结构体包含 7 个 HashMap（local_vars/local_mixins/local_functions/forwarded_vars/forwarded_mixins/forwarded_functions/global_writes）+ `parent: Option<Rc<Scope>>`
- `Env` 方法全部 `self -> Self`（链式）
- `eval_xxx` 方法接收 `Env`（move），返回 `(Vec<CssNode>, Env)`
- 只读辅助方法（`call_function` / `bind_params` / `load_module`）保持 `&Env`
- 作用域进出用 `enter_scope()` / `exit_scope()` — 零 clone（`Rc::clone` 原子计数器递增）
- 写操作通过 `Rc::try_unwrap` 获取 scope 所有权（引用计数为 1 时零 clone）
- 变量查找沿 scope 链向上搜索（`Scope::lookup`）
- flow control（`@if`/`@for`/`@each`）不创建新 scope — 符合 SCSS 规范
- **禁止** `env.clone()`（除 `@content` 上下文快照）
- **禁止** `Rc::make_mut`
- 源文件：`scope.rs`（Scope 结构 + 方法）、`env.rs`（Env/ModuleExports/MixinDef/FunctionDef 类型定义）、`env_impl.rs`（Env 方法实现）

## 验证清单（修复后必跑）

```bash
cargo test --test compile_test    # 57 个（含 14 个 CSS Color Level 4 色彩空间测试）
cargo test --test stage_test      # 10 个
cargo test --test ast_test        # 8 个
cargo test --test common_test     # 5 个
cargo test --test interp_test     # 15 个
cargo test --test bs_spec -- --nocapture    # 15 个
cargo test --test ep_full -- --nocapture    # 121 个（约 38 秒）
cargo test --test default_config_test -- --test-threads=1  # 9 个

# sass-spec 全量统计（约 4 分钟）
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture

# sass-spec 全量统计 + OTel 追踪（输出 span 到 stdout）
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --features otel --test sass_spec_full -- --nocapture

# 全量失败原因导出 JSON（~87 秒，输出 tests/sass-spec-failures.json）
cargo test --test failures_json -- --nocapture
```

**通过标准**：57/57 + 8/8 + 8/8 + 5/5 + 15/15 + 15/15 + 121/121 + 9/9
**sass-spec 基线**：7444/11869 = 62.7%（含 color 目录，跳过 libsass 不支持目录）
**ep_full**：121/121 = 100%
**颜色测试**：已跳过（防止无限修复循环，需 `--ignored` 手动触发）

### 颜色测试跳过策略

颜色相关 spec 测试已全部加入跳过列表，防止在非颜色任务中反复触发颜色测试失败导致无限修复循环：

- **SKIP_DIRS**（`tests/spec_manifest.rs`）：`core_functions/color` + `values/colors` — 全量统计和诊断自动跳过
- **#[ignore] 测试函数**：
  - `sass_spec_full::test_core_functions_subdirs` — 17 个颜色子目录统计
  - `cf_diag::diag_color` — core_functions/color 诊断
  - `cf_diag::diag_values_colors` — values/colors 诊断
  - `cf_color::color_error_patterns` — 颜色错误模式统计
  - `minimize::minimize_color_error` — 颜色错误最小化
- **手动触发颜色测试**：`cargo test --test <file> -- --ignored`

## HRX 解析架构（内联 hrx_support 模块）

sasspile 测试模块通过 `tests/hrx_support.rs` 内联 HRX 解析，**不依赖外部 crate**：

- `hrx_support::parse_hrx(content)` → `Result<HrxArchive, String>`
- `hrx_support::Vfs::from_archive(&archive)` → `Vfs`（虚拟目录树）
- `hrx_support::parse_hrx_to_cases(content, hrx_rel_path)` → `Vec<HrxCase>`（高级 API，路径加 HRX 名作前缀）
- 测试代码**不再按 `===` 分组隔离**——所有条目共享同一个 VFS，路径加 HRX 目录前缀，使 `@use` 跨组引用能正确解析
- **颜色相关目录自动注入 `_utils.scss`**：使用 `OnceLock` 缓存物理文件 `sass-spec/spec/core_functions/color/_utils.scss`，首次访问时读取并注入到 VFS 文件列表，解决跨组 `@use 'core_functions/color/_...'` 路径解析问题
- 共享模块被 9 个测试文件引用：`sass_spec_full.rs`、`cf_diag.rs`、`css_diag.rs`、`expr_diag.rs`、`sass_spec.rs`、`diag_detail.rs`、`minimize.rs`、`cf_color.rs`、`diag_directives.rs`

## 🔄 Git 规范

| 规则 | 说明 |
|------|------|
| 推送方式 | SSH：`git push origin main`（remote 名为 `origin`，SSH 地址 git@github.com） |
| Commit 格式 | `feat: 描述 — 总计 N/M` |
| 只提交不推送 | commit 后必须等用户确认再推送 |
| 提交后同步 | 每次提交后必须 `codegraph sync`（确保代码导航索引与最新代码一致） |

## OpenSpec 归档

已归档变更存储在 `openspec/changes/archive/` 目录。最近归档：
- **failures-json**（2026-09-09）：全量失败原因 JSON 导出 — 新建 `tests/failures_json.rs`，运行全部 sass-spec case（11869 个，~87 秒），将每个失败的完整 expected/actual/error 写入 `tests/sass-spec-failures.json`（含 metadata + failures[] + by_dir + by_type 聚合）— 用于回归检测和失败模式分析
- **clippy-cleanup**（2026-09-09）：全量 clippy 清理 — unwrap→expect、eprintln→tracing::error、float_cmp→abs<EPSILON、format! 内联变量、let...else 重写、items_after_statements 修复 — cargo clippy 零错误，核心测试全通过
- **cf-noncolor-boost**（2026-09-08）：core_functions 非 color 子域修复 — list join/set-nth/zip/is-bracketed 修复 (+48)、math sin/cos/tan 角度单位转换 (+13)、meta module_exports 全覆盖 (+120) — sass-spec 7144→7365 (+221)
- **color-adjust-units**（2026-09-06）：CIE+Modern RGB 颜色 adjust/change/scale percent 单位处理 — 新增 cie_channel 提取器（区分 unitless n 与 n%），CIE 各通道正确 max 值（Oklch/Oklab/Lch/Lab），Modern RGB 统一使用 cie_channel(max=1.0)，sass-spec 6426→6695 (+269)
- **color-level4-complete**（2026-09-06）：CSS Color Level 4 测试工具链修复（.sass 过滤 + _utils.scss OnceLock 注入）+ 14 个 lab/lch/oklab/oklch 构造序列化测试 — 核心测试 100/100 全通过，sass-spec 6264→6426 (+162)
- **sass-spec-boost**（2026-09-05）：color/to_space NaN 处理 + HWB/HSL 序列化 + scale/change HWB 通道扩展 — 202/202 核心测试全通过，sass-spec 6205→6264 (+59)
- **sass-spec-completeness**（2026-09-05）：CSS at-rules 全链路（@keyframes/@font-face/@page/@charset/@namespace/@layer/@container）+ meta 反射修复 + 颜色算法精度 + selector-replace compound-level subset matching — 202/202 核心测试全通过，sass-spec 6123→6205 (+82)
- **functional-cleanup**（2026-09-04）：函数式风格全量清理 — else-if 链→match（81处）、for+push→迭代器链（76处）、if-let 链→apply_kw 链式、&mut 参数→move 语义 — 105/105 核心测试通过，sass-spec 3366/5624 维持基线
- **selector-ast**（2026-09-04）：选择器 AST + calc 简化函数式重构 — 226/226 全通过，sass-spec 3327→3366 (+39)
- **scope-chain-arch**（2026-09-03）：Scope Chain 作用域管理架构 — 202/202 全通过，sass-spec 3324→3327
- **otel-integration**（2026-09-02）：OpenTelemetry 0.32 集成
- **hrx-auditor-removal**（2026-09-02）：移除 hrx-auditor 外部依赖，内联 hrx_support.rs
- **calc-simplification**（2026-09-01）：calc 表达式简化 + CSS round/mod/rem — 2902→3068 (+166)

更早的归档记录详见 `openspec/changes/archive/` 目录。

## 内建函数注册架构（builtin-dispatch-macro）

- **sasspile-macros** proc-macro crate（workspace 成员）：通过 `#[derive(BuiltinRegistry)]` 将三处重复 match 合并为单一数据源
- 依赖：syn 3.0 + quote + proc-macro2（未用 darling，改用 syn 3.0 原生 `parse_nested_meta`）
- 7 个结构体：MathBuiltins / StringBuiltins / MapBuiltins / ListBuiltins / ColorBuiltins / MetaBuiltins / SelectorBuiltins
- 宏自动生成：`module_builtin_name`（模块限定名 → 全局名）、`is_known_builtin`（已知函数检查）、`dispatch_builtin_module`（模块分派）
- `#[builtin(module = "math", dispatch = "math")]` 声明模块名和分派目标
- `#[builtin(alias = "math.div")]` 声明字段别名（模块限定名）
- 宏自动生成 `module.kebab-case` 默认别名
- `dispatch = "none"` 表示只参与名称映射不分派（meta 模块）
- 手工保留：rgba/rgb/darken/lighten/mix 的分派和 is_known_builtin

## 颜色系统架构

sasspile 颜色系统基于 `ColorSpace` 枚举（17 种色彩空间）+ `ColorOutput` 枚举（3 种输出模式）实现 CSS Color 4 完整支持。

**类型定义**（`src/parse/ast/color_types.rs`）：

| 类型 | 说明 |
|------|------|
| `ColorSpace` | 色彩空间标识（Rgb/Srgb/DisplayP3/A98Rgb/ProphotoRgb/Rec2020/XyzD65/XyzD50/Hsl/Hwb/Lab/Lch/Oklab/Oklch 等） |
| `ColorOutput` | 输出模式（Auto / RgbExplicit / RgbPercent） |
| `ChannelSet` | 通道名分组（Hsl/Hwb/Rgb/Lab/Lch/Oklab/Oklch/Xyz） |
| `Color` | `{ space, channels[3], alpha, output, legacy_rgb[3] }` |

**序列化格式示例**：

| 空间 | 输出示例 | 备注 |
|------|----------|------|
| `Rgb` (Auto) | `#ff0000`, `red`, `rgba(0,0,0,0.5)` | hex / 命名色优先 |
| `Hsl` | `hsl(120, 50%, 50%)` | NaN hue → 0 |
| `Hwb` | `hwb(0 30% 40%)` / `hsl(...)` | 全有效值时规范化为 HSL |
| `Lab` | `lab(50% 40 59.5)` | L% a b |
| `Lch` | `lch(50% 50 270deg)` | chroma=0 时 hue → `none` |
| `Oklab` | `oklab(59% 0.1 0.1)` | L% a b（L 从 0-1 转为 0%-100%） |
| `Oklch` | `oklch(70% 0.1 180deg)` | chroma=0 时 hue → `none` |
| `DisplayP3` | `color(display-p3 1 0 0)` | 现代RGB空间 |
| `Srgb` / `SrgbLinear` | `color(srgb 1 0 0)` | |
| `XyzD65` / `XyzD50` | `color(xyz 0.5 0.5 0.5)` | |

**关键规则**：
- `hsl()`/`hsla()` 创建的颜色保留 HSL 格式输出
- `darken`/`lighten`/`saturate`/`adjust-hue`/`complement`/`invert`/`grayscale` 等操作函数用 `RgbPercent` 输出
- `adjust-color`/`change-color`/`scale-color` 修改 HSL/HWB 参数时用 `RgbPercent`，纯 RGB 参数时用 `Auto`
- **CSS Color 4 现代空间**：`color_conv.rs` 使用 W3C 有理数分数矩阵（sRGB↔XYZ/Lab/Oklab），`color_adjust.rs` 支持现代空间 adjust/change/scale，`color_gamut.rs` 实现 clip + local-minde 色域映射

**格式化辅助函数**（`src/parse/ast/color_fmt.rs`）：
- `format_hue(h)` — hue 截断到 10 位小数，NaN → "none"
- `format_pct(v)` — 百分比格式化（0-1 → 0%-100%）
- `format_num(n)` — 通用浮点格式化，NaN → "none"

## 🤖 AI 代码生成防抖规范

> 防止 AI 在长文件中反复修改、上下文丢失、产生矛盾代码

### 行为准则

1. **先读后写** — 修改前必须先读取文件完整内容，理解现有结构
2. **一次一事** — 单次任务只做一种改动（重构 / 修 bug / 加特性分开提交）
3. **锚点保留** — 保留现有分区注释格式
4. **小步快跑** — 每次生成代码控制在 **50 行以内**的 diff
5. **上下文锚定** — 在关键代码段添加 `// ANCHOR: <name>` 注释，便于 AI 精确定位
6. **测试先行** — 修改核心逻辑前，先添加对应测试用例

### 抖动前兆检测

出现以下任一情况，**立即停止当前任务**并通知用户：

- 修改波及 **3 个以上**不相关函数/结构体
- 同一行被反复修改 **2 次以上**
- 新增代码与现有枚举/trait 定义矛盾
- 生成代码超过目标文件行数限制的 **80%**
- AI 重复生成相同或相似的代码片段
- 连续 **2 次**在同一个函数中添加 `clone()`
- 把 `self -> Self` 改回 `&mut self`
- 用 `for + push` 替换已有的 `map/collect`
- 在纯函数中引入 `&mut` 参数
- 用 `if-else` 链替换已有的 `match`

## 🔧 高效工具优先（违反 = 任务失败）

> **核心原则**：禁止使用低效手动方式（grep、sed、逐行命令行分析、重复打印查询）。所有任务必须使用已建立的高效工具链。

### 工具链总览

| 类别 | 工具 | 触发场景 |
|------|------|----------|
| 代码统计 | `tests/sass_spec_stats.rs` | sass-spec 通过率报告、基线对比 |
| 失败诊断 | `tests/css_diag.rs`、`tests/expr_diag.rs`、`tests/cfs_diag.rs`、`tests/diag_directives.rs` | 定位具体失败 case |
| 失败导出 | `tests/failures_json.rs` → `tests/sass-spec-failures.json` | 全量失败原因结构化导出（含完整 expected/actual） |
| 代码查询 | `codegraph callers/impact/node/explore/callees` | 调用链分析、影响范围 |
| 链路追踪 | `RUST_LOG=trace --features otel` | 跨函数/跨阶段 bug 定位 |
| 脚本处理 | `rust-script` 或 Rust test | 任何脚本/数据处理任务 |

### sass-spec 诊断工具

```bash
# 1. 全量统计（~4 分钟，后台运行）
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture > /tmp/sass-spec-full.log 2>&1 &

# 2. 生成 MD 报告 + 基线对比
cargo test --test sass_spec_stats -- --nocapture

# 3. 保存当前为新基线
BASELINE=1 cargo test --test sass_spec_stats -- --nocapture

# 4. 导出全量失败原因 JSON（~87 秒）
cargo test --test failures_json -- --nocapture
# 输出: tests/sass-spec-failures.json（含完整 expected/actual/error）
```

### failures_json 输出格式

`tests/sass-spec-failures.json` 包含：
- `metadata` — timestamp、total_cases、pass、fail、skip
- `failures[]` — 每条含 `id`、`dir`、`type`（DIFF/ERR/ERR_EXP_OK）、完整 `expected`/`actual`/`error`
- `by_dir` — 按目录聚合的各类型计数
- `by_type` — DIFF/ERR/ERR_EXP_OK 总计数

**用途**：回归检测（对比两次 JSON）、失败模式分析、修复优先级排序。

### 失败定位工具

```bash
# CSS 失败（含 selector/extend）
RUST_LOG="css_diag=info" cargo test --test css_diag

# 表达式失败
RUST_LOG="expr_diag=info" cargo test --test expr_diag

# 指令失败
RUST_LOG="diag_directives=info" cargo test --test diag_directives

# core_functions 失败
RUST_LOG="cfs_diag=info" cargo test --test cfs_diag
```

### 禁止模式

| 禁止 | 替代 |
|------|------|
| `grep -r "xxx" /tmp/log \| head -20` 反复查询 | 写 Rust test 解析日志，一次生成报告 |
| `codegraph callers` 重复 5 次不同角度 | `codegraph explore` 一次探索完整上下文 |
| 手动逐行翻日志 200 行 | `cargo test --test xxx_diag` 输出结构化失败汇总 |
| python3 处理数据 | `rust-script -e` 或新建 test 文件 |
| bash 循环分析 | Rust test 中用迭代器链处理 |

### 工作流

1. **统计需求** → `sass_spec` 后台运行 → `sass_spec_stats` 生成报告
2. **定位失败** → 运行对应 `*_diag` 测试获取结构化输出
3. **全量导出** → `failures_json` 生成 `sass-spec-failures.json`（回归检测/修复对比）
4. **分析影响** → `codegraph callers/impact/node`
5. **Bug 追踪** → `RUST_LOG=trace --features otel` + `#[instrument]`
6. **数据转换** → 写 Rust test 或 `rust-script`

## 🔍 CodeGraph 优先

查询调用链、影响分析、代码流向时，**使用 CodeGraph CLI**（优先于 LSP 或手动阅读）：

### 索引管理

```bash
codegraph init [path]          # 初始化项目索引
codegraph sync [path]          # 增量同步（最常用）
codegraph index [path]         # 全量重建
codegraph status [path]        # 索引统计
codegraph files                # 项目文件结构
```

### 代码查询

```bash
codegraph callers <symbol>     # 谁调了这个函数
codegraph callees <symbol>     # 这个函数调用了什么
codegraph impact <symbol>      # 修改某符号的影响
codegraph affected [files...]  # 受影响的测试文件
codegraph node [name]          # 符号定义 + 调用链
codegraph explore <query...>   # 自然语言代码探索
codegraph query <search>       # 搜索符号
```

### 工作流

1. 代码变更后 → `codegraph sync`（更新索引）
2. 查找调用者 → `codegraph callers fn`
3. 影响分析 → `codegraph impact fn`
4. 探索不熟悉代码 → `codegraph explore "auth flow"`
5. 符号定义 → `codegraph node parse_expr`
6. 受影响测试 → `codegraph affected src/parser.rs`

## 参考文档（需要时查阅）

- **代码导航**：CodeGraph（动态查询，优先）— `codegraph callers/impact/node/explore`
- **综合开发技能**：根目录 `skill.md`（编译管线 + 内建函数 + CSS 序列化 + 调试追踪）
- **函数式 Rust**：`.claude/skills/functional-rust/SKILL.md`（优先级表 + 正反对比 + 反模式检测）
- **调试技能**：`.claude/skills/tracing-debug/SKILL.md`
- **OpenSpec 工作流**：`.claude/skills/openspec-*/SKILL.md`

## 文件解析架构（file_resolver.rs）

- `file_resolver.rs` 承载文件路径解析逻辑：`resolve_file`、`try_resolve_dir`、`check_resolve_ambiguity`
- `check_resolve_ambiguity` 检测四种文件冲突场景：
  1. partial vs non-partial（`_file.scss` 和 `file.scss` 同时存在）
  2. extension 冲突（`file.scss` 和 `file.sass` 同时存在）
  3. index 冲突（`dir/_index.scss` 和 `dir/index.scss` 同时存在）
  4. import-only 冲突（`file.import.scss` 和 `file.import.sass`）
- `module_helpers.rs` 统一承载 `bind_exports`（含 values_eq + Display 后备检查）、`merge_module_cache`、`BindMode`、`FilterConfig` 等 pub(crate) 辅助函数

## ✅ 自检清单

每次任务完成后：

- [ ] 未使用 Python
- [ ] 所有输出用 tracing 宏（无 println!/eprintln!）
- [ ] 测试在 tests/ 目录（无 inline #[cfg(test)]）
- [ ] 跨函数/管道使用 tracing span（或 `#[instrument]`）
- [ ] span 字段用 `?`/`%` sigil（非 `&format!(...)`）
- [ ] async 代码不用 `Span::enter`（用 `#[instrument]` 或 `.instrument()`）
- [ ] 无 `unwrap()`（用 `?`/`expect()`/`unwrap_or()`）
- [ ] 无 `clone()` 满天飞（先理解所有权设计）
- [ ] 无 `todo!()`/`unimplemented!()` 不标注 TODO
- [ ] 公开 API 有 `///` 文档注释
- [ ] 单文件 ≤ 500 行
- [ ] 集合变换用 `map/filter/collect` 而非 `for + push`
- [ ] 枚举分派用 `match` 而非 `if-else` 链
- [ ] 错误传播用 `?` 而非 `match ... Err(e) => return`
- [ ] 状态变更返回新值（`self -> Self`）而非 `&mut self`
- [ ] 管线阶段消费 `self`（类型状态机）而非 `&self` + clone
- [ ] 累积操作用 `try_fold` / `fold` 而非可变 `Vec` + push
- [ ] 分流用 `partition` 而非两个 `Vec` + for + if
- [ ] 调试遵循 4 步协议（如果是 bug 修复）
- [ ] OTel 追踪可用：`cargo test --features otel` 输出 span 正常
- [ ] CodeGraph 用于代码查询
- [ ] 使用高效工具链（sass_spec_stats / *_diag / codegraph / OTel），未手动 grep/逐行分析
- [ ] Commit 等用户确认后再推送
