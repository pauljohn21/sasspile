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
RUST_LOG=trace cargo test test_name 2>&1 | tee /tmp/trace.log
```

保留 trace 输出作为**证据**。

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

## ⛔ sasspile 特定规则

1. **禁止 #[cfg(test)] 内联测试**：所有测试放在 tests/ 目录，src/ 保持纯生产代码。
2. **修复 bug 前必须 tracing 追踪**：RUST_LOG=info/debug cargo test 查看完整链路。

## 会话开始检查清单

每次新会话，先确认：
- [ ] 读 workspace 规则（本文件）
- [ ] 读用户规则（user_rules 部分）
- [ ] 检查是否有相关记忆需要加载

## 项目核心

sasspile 是纯 Rust 函数式 SCSS 编译器。架构：

```
Source → Lexer → Parser → Evaluator → Serializer → CSS
(lex/)   (parse/)  (eval/)     (css/)
```

### 函数式管线（链式调用 + move 语义）

入口 `lib.rs` 全部链式调用，数据通过 move 语义流过管线：

```rust
// 字符串编译
Source::new(input.to_string())
    .lex()?
    .parse()?
    .evaluate()?
    .serialize(style)
    .into_string()

// 文件编译
Source::from_file(path)?
    .with_load_paths(load_paths)
    .lex()?
    .parse()?
    .evaluate()?
    .serialize(style)
    .into_string()
```

### Stage 类型状态机

每个阶段是一个新类型，阶段转换是该类型的方法：
- `Source` — 携带 `text` + `base_path` + `load_paths`
- `Lexed` — 携带 `tokens` + 透传 `base_path` + `load_paths`
- `Parsed` — 携带 `ast` + `base_path` + `load_paths`，`evaluate()` 内部构建 `Env`
- `Evaluated` — 携带 `Vec<CssNode>`
- `Serialized` — 最终 CSS 字符串

### Env 设计（move 语义，零 clone）

- `Env` 方法全部 `self -> Self`（链式）
- `eval_xxx` 方法接收 `Env`（move），返回 `(Vec<CssNode>, Env)`
- 只读辅助方法（`call_function` / `bind_params` / `load_module`）保持 `&Env`
- **禁止** `env.clone()`（除 `@content` 上下文快照）
- **禁止** `Rc::make_mut`（字段已恢复为 `HashMap`）

## 验证清单（修复后必跑）

```bash
cargo test --test compile_test    # 43 个
cargo test --test stage_test      # 10 个
cargo test --test ast_test        # 8 个
cargo test --test common_test     # 5 个
cargo test --test interp_test     # 15 个
cargo test --test bs_spec -- --nocapture    # 15 个
cargo test --test ep_full -- --nocapture    # 121 个（约 38 秒）
cargo test --test default_config_test -- --test-threads=1  # 9 个

# sass-spec 全量统计（约 70 秒）
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture
```

**通过标准**：43/43 + 10/10 + 8/8 + 5/5 + 15/15 + 15/15 + 121/121 + 9/9
**sass-spec 基线**：3068/5362 = 57%（VFS + `===` 分组隔离，跳过 libsass/color/colors 目录，calc 简化 + CSS round/mod/rem 函数 + 括号去除后 +166）
**@directives 子目录**：forward 76%，import 32 FAIL（conflict 5/5 修复，pending_config 架构生效）
**ep_full**：121/121 = 100%（file_resolver.rs 拆分 + module_helpers 统一后无回归）
**core_functions/color 子目录**：已跳过（防止无限修复循环，需 `--ignored` 手动触发）

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

## HRX 解析架构（hrx-auditor 集成）

sasspile 通过 dev-dependency 引用 `hrx-auditor` crate（路径 `../scss-rust`），直接使用其 VFS + parser：

```toml
[dev-dependencies]
hrx-auditor = { path = "../scss-rust" }
```

- `hrx_auditor::parser::parse_hrx(content)` → `HrxArchive`
- `hrx_auditor::vfs::Vfs::from_archive(&archive)` → `Vfs`（虚拟目录树）
- 测试代码按 `===` 分隔符将 entries 分成独立组，每组构建自己的 VFS，正确隔离不同测试组的文件
- 已迁移全部 8 个测试文件：`sass_spec_full.rs`、`cf_diag.rs`、`css_diag.rs`、`expr_diag.rs`、`sass_spec.rs`、`diag_detail.rs`、`minimize.rs`、`cf_color.rs`

## 🔄 Git 规范

| 规则 | 说明 |
|------|------|
| 推送方式 | SSH：`git push origin main`（remote 名为 `origin`，SSH 地址 git@github.com） |
| Commit 格式 | `feat: 描述 — 总计 N/M` |
| 只提交不推送 | commit 后必须等用户确认再推送 |
| 提交后同步 | 每次提交后必须 `codegraph sync`（确保代码导航索引与最新代码一致） |

## OpenSpec 归档

- 已归档变更存储在 `openspec/changes/archive/` 目录
- **rc-env-perf**（2026-08-22 归档）：Rc COW 性能方案（已被 mut-env-refactor 取代）— Env 从 im::HashMap 改为 Rc<HashMap> COW，后因模仿 GC 模式被弃用
- **mut-env-refactor**（2026-08-22 归档）：Env move 语义重构 + 函数式链式调用 — lib.rs 入口改链式 Source→Lexed→Parsed→Evaluated→Serialized，15 个 eval_xxx 改 Env（move），消除 env.clone() 和 Rc::make_mut()，stage 管线携带 base_path + load_paths — 202/202 全通过
- **fix-top-level-declaration**（2026-08-22 归档）：顶层 CSS 声明检测 — eval_node 的 Node::Decl 分支增加 current_selector 检查
- **spec-pass-rate-boost**（2026-08-21 归档）：参数验证修复 + meta 模块功能 + error 检测 + values/css 深度修复 — 5 个 spec 已同步到 `openspec/specs/`
- **builtin-dispatch-macro**（2026-08-21 归档）：派生宏重构内建函数注册 — 1 个 spec（`builtin-registry`）已同步到 `openspec/specs/`
- **fix-forward-use-conflict**（2026-08-21 归档）：local/forwarded 双层结构 + bind_exports 重构 + @forward show/hide 过滤 + @import 内联合并 — ep_full 10/121→121/121
- **directives-100**（进行中）：文件歧义检测增强（partial/extension/index/import-only 四种冲突）+ module_helpers 统一 + .sass 测试修复 — conflict 5/5 修复, import 37→32 FAIL
- **chain-reaction**（2026-08-31 归档）：全面链式反应重构 — eval_nodes/eval_for/eval_each 用 try_fold，hoist_css_imports 用 partition，eval_rule 用 RuleBuilder+fold，flatten_nodes 用 flat_map+partition，merge_at_rules 用 fold，Evaluated::serialize 改为 self 消费 — 202/202 全通过，sass-spec 2828→2902 (+74)
- **calc-simplification**（2026-09-01 归档）：calc 表达式简化 + CSS round/mod/rem 函数 — simplify_calc 支持纯数字/常量(pi/e)/同单位算术/科学计数法/嵌套 min/max 简化，strip_parens 去除多余括号，remove_unnecessary_parens 去除乘除法括号，CSS round() 四种策略(nearest/up/down/to-zero)+单位转换，CSS mod()/rem() floored/truncated modulo，calc 函数名大小写不敏感，math 函数 Calc 参数透传 — 2902→3068 (+166)，1 个 spec（`calc-simplification`）已同步到 `openspec/specs/`
- **fix-default-config-validation**（2026-08-31 归档）：@forward 链 !default 配置验证 — eval_forward 回传 consumed_config 正确处理 as 前缀映射，config_pairs 仅传递 with 声明变量，load_module 区分 @use（验证）和 @forward（不验证）场景 — 1 个 spec（`use-with-validation`）已同步到 `openspec/specs/`
- **fix-interp-eval**（2026-08-31 归档）：插值求值架构重构 — Value::Interp 从 String 改为 Vec<InterpSegment> 保留表达式与文本边界，parser parse_interp_adjacent 方法拼接相邻片段，eval_interp_segments 逐片段求值 — 1 个 spec（`interp-eval`）已同步到 `openspec/specs/`，15 个 interp_test 全通过

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

sasspile 颜色系统基于 `ColorFormat` 枚举追踪颜色创建方式，影响序列化输出：

| 格式 | 用途 | 示例 |
|------|------|------|
| `Auto` | hex / 命名颜色 / rgba（默认） | `#ff0000`, `red`, `rgba(0,0,0,0.5)` |
| `Rgb` | rgb(r,g,b) / rgba(r,g,b,a)（不转 hex） | `rgb(255, 0, 0)` |
| `RgbPercent(h,s,l)` | rgb(r%,g%,b%) 百分比输出（HSL 操作结果） | `rgb(72%, 0%, 0%)` |
| `Hsl(h,s,l)` | hsl(h,s%,l%) / hsla(...)（保留原始 HSL） | `hsl(120, 50%, 50%)` |
| `Hwb(h,w,b)` | hwb(h w% b%) / hwb(h w% b% / a) | `hwb(0 30% 40%)` |
| `Lab(l,a,b)` | lab(L% a b)（CSS Color 4 Lab） | `lab(50% 40 59.5)` |
| `Lch(l,c,h)` | lch(L% C Hdeg)（CSS Color 4 LCH） | `lch(50% 50 270)` |
| `Oklab(l,a,b)` | oklab(L% a b)（CSS Color 4 OkLab） | `oklab(59% 0.1 0.1)` |
| `Oklch(l,c,h)` | oklch(L% C Hdeg)（CSS Color 4 OKLCH） | `oklch(70% 0.1 180)` |
| `DisplayP3(r,g,b)` | color(display-p3 r g b) | `color(display-p3 1 0 0)` |
| `Srgb(r,g,b)` | color(srgb r g b) | `color(srgb 1 0 0)` |
| `XyzD65(x,y,z)` / `XyzD50(x,y,z)` | color(xyz r g b) / color(xyz-d50 r g b) | `color(xyz 0.5 0.5 0.5)` |

**关键规则**：
- `hsl()`/`hsla()` 创建的颜色保留 HSL 格式输出
- `darken`/`lighten`/`saturate`/`adjust-hue`/`complement`/`invert`/`grayscale` 等操作函数用 `RgbPercent` 输出
- `adjust-color`/`change-color`/`scale-color` 修改 HSL/HWB 参数时用 `RgbPercent`，纯 RGB 参数时用 `Auto`
- **CSS Color 4 现代空间**：`color_conv.rs` 使用 W3C 有理数分数矩阵（sRGB↔XYZ/Lab/Oklab），`color_adjust.rs` 支持现代空间 adjust/change/scale，`color_gamut.rs` 实现 clip + local-minde 色域映射
- 依赖 `color` crate v0.3 提供色彩空间转换参考

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

## 📝 编码规范速查

### 命名

| 类型 | 规范 | 示例 |
|------|------|------|
| 函数/变量 | snake_case | `fn parse_input()` |
| 类型/结构体 | CamelCase | `struct HttpClient` |
| 常量 | SCREAMING_CASE | `const MAX_RETRIES: u32 = 3;` |
| 模块 | snake_case | `mod user_service;` |

### 转换方法命名

| 前缀 | 语义 | 开销 | 示例 |
|------|------|------|------|
| `as_` | 廉价引用转换 | `&T` | `as_str()` |
| `to_` | 昂贵转换 | 分配 | `to_string()` |
| `into_` | 消耗所有权 | move | `into_vec()` |

### 已弃用 → 推荐

| 已弃用 | 推荐 | 起始版本 |
|--------|------|---------|
| `lazy_static!` | `std::sync::OnceLock` | 1.70 |
| `once_cell::Lazy` | `std::sync::LazyLock` | 1.80 |
| `std::sync::mpsc` | `crossbeam::channel` | — |
| `std::sync::Mutex` | `parking_lot::Mutex` | — |
| `failure` / `error-chain` | `thiserror` / `anyhow` | — |
| `try!()` | `?` 操作符 | 2018 |

### 文档注释

- 公开 API 必须有 `///` 文档注释
- 模块用 `//!` 文档注释

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

- **代码导航**：CodeGraph（动态查询，优先）/ `docs/CODE_INDEX.md`（静态参考）
- **综合开发技能**：根目录 `skill.md`（编译管线 + 内建函数 + CSS 序列化 + 调试追踪）
- **函数式 Rust**：`.claude/skills/functional-rust/SKILL.md`（优先级表 + 正反对比 + 反模式检测）
- **调试技能**：`.claude/skills/tracing-debug/SKILL.md`
- **OpenSpec 工作流**：`.claude/skills/openspec-*/SKILL.md`
- **源文件结构**：见 `docs/CODE_INDEX.md`

## 文件解析架构（file_resolver.rs）

- `file_resolver.rs` 承载文件路径解析逻辑：`resolve_file`、`try_resolve_dir`、`check_resolve_ambiguity`
- `check_resolve_ambiguity` 检测四种文件冲突场景：
  1. partial vs non-partial（`_file.scss` 和 `file.scss` 同时存在）
  2. extension 冲突（`file.scss` 和 `file.sass` 同时存在）
  3. index 冲突（`dir/_index.scss` 和 `dir/index.scss` 同时存在）
  4. import-only 冲突（`file.import.scss` 和 `file.import.sass`）
- `module_helpers.rs` 统一承载 `bind_exports`（含 values_eq + Display 后备检查）、`merge_module_cache`、`BindMode`、`FilterConfig` 等 pub(crate) 辅助函数

## 常用命令（需要时查阅）

```bash
# 追踪错误链路
RUST_LOG=info cargo test --test compile_test <test_name> -- --nocapture
RUST_LOG="sasspile::color=trace" cargo test --test compile_test -- --nocapture

# sass-spec 诊断
cargo test --test cf_diag diag_<subdir> -- --nocapture
RUST_LOG="minimize=info" cargo test --test minimize minimize_<subdir>_error -- --nocapture

# CodeGraph 查询（优先使用）
codegraph sync                # 同步索引（每次提交后必跑）
codegraph impact <symbol>      # 影响分析
codegraph callers <function>   # 谁调了这个函数
codegraph explore "query"      # 自然语言探索
codegraph node <symbol>       # 查看符号源码 + 调用链路

# Rust 通用命令
cargo check                              # 快速检查
cargo clippy --workspace                 # 全 workspace clippy
cargo clippy -- -W clippy::pedantic      # 严格 clippy
cargo test                               # 运行所有测试
cargo nextest run                        # 使用 nextest（更快）
cargo build --release                    # 发布构建
cargo doc --open                         # 生成并打开文档
cargo fmt                                # 格式化所有代码
cargo fmt -- --check                     # 检查格式（CI 用）
cargo tree                               # 依赖树
```

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
- [ ] CodeGraph 用于代码查询
- [ ] Commit 等用户确认后再推送
