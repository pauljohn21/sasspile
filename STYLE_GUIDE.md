# sasspile 风格指南

本文档定义 sasspile 项目（`src/` + `tests/`）的统一代码风格规范。

所有贡献者（含 AI 辅助生成代码）应遵循本文档。

---

## 1. Module Header 模板

每个 `.rs` 文件必须以 `//!` module-level doc comment 开头。

### 1.1 src/ 模块模板

```rust
//! —— ModuleName ——
//!
//! 概要：一句话说明模块职责。
//!
//! ## 核心概念
//! - 关键类型/算法
//! - 与上下游模块的关系
//!
//! ## 示例（可选，简单模块可省略）
//! ```rust
//! use ...;
//! ```
```

**示例**（`src/eval/reactor.rs`）：

```rust
//! —— Reactor 编译管线 ——
//!
//! 概要：类型状态机模式封装编译全过程（lex → parse → evaluate → serialize）。
//!
//! ## 核心概念
//! - `Reactor<S>` 通过泛型参数 S 编码管线阶段
//! - 消费-返回 API 保证无隐式共享状态
//! - IO 通过全局 tokio runtime + block_on 异步化
//!
//! ## 管线流程
//!
//! ```text
//! Reactor::new(source).lex()?.parse()?.evaluate()?.serialize(style).finish()
//! ```
```

### 1.2 tests/ 文件模板

```rust
//! —— Test Description ——
//!
//! 概要：一句话说明测试覆盖内容。
//!
//! ## 覆盖场景
//! - 场景 A
//! - 场景 B
//!
//! ## sass-spec 参照（如适用）
//! - `core_functions/list/join.hrx` — 命名参数校验
//! - `directives/use/` — 模块加载
```

**示例**（`tests/selector_unify_test.rs`）：

```rust
//! —— 选择器 unify/extend/replace 算法测试 ——
//!
//! 概要：验证选择器 unify/extend/replace 算法在各种场景下的正确性。
//!
//! ## 覆盖场景
//! - unify：class/id/type/combinator 统一
//! - extend：parent/grandparent replacement、leading combinator
//! - replace：simple/no-match 替换
//! - parser error detection：未闭合属性选择器
//!
//! ## sass-spec 参照
//! - `core_functions/selector/extend.hrx` — extend 算法
//! - `core_functions/selector/unify.hrx` — unify 算法
```

### 1.3 免 header 情形

以下文件可省略完整 header，仅保留一句话概要：

- `tests/common/mod.rs` — 共享辅助函数
- `tests/hrx_support.rs` — HRX 解析基础设施
- `tests/spec_manifest.rs` — sass-spec 文件清单

---

## 2. Section Divider 规范

### 2.1 字符

统一使用 `─` (U+2500, BOX DRAWINGS LIGHT HORIZONTAL)。

**禁止**：
- `═` (U+2550) — 双线，仅在旧文件中使用，需替换
- `—` (U+2014, em dash) — 双字符混淆，需替换

### 2.2 格式

```rust
// ─── Section Name ───────────────────────────────────────────
```

- 左对齐：`// ─── ` (5 字符前缀)
- 右对齐：补齐到 78 列
- 两侧各一个空格分隔名称

### 2.3 对齐计算

```
// ─── Name ─────────────────────────────────────────────────
123456789...
```

总长度 = 78 字符。前缀 `// ─── ` = 7 字符，后缀 `───` 补齐。

**实现方法**：Name 部分前后各一个空格，剩余用 `─` 补齐。

### 2.4 示例

```rust
// ─── 数据结构 ─────────────────────────────────────────────────────────────

// ─── 解析 ──────────────────────────────────────────────────────────────

// ─── 报告生成 ──────────────────────────────────────────────────────────
```

---

## 3. 函数声明格式

### 3.1 测试函数

统一使用展开格式：

```rust
#[test]
fn test_name() {
    ...
}
```

**禁止**紧凑同行格式：

```rust
// ❌ 禁止
#[test] fn test_name() { ... }
```

即使是单行函数体也展开：

```rust
// ✅ 正确
#[test]
fn diag_list() {
    diag("core_functions/list", 15);
}
```

### 3.2 普通函数

```rust
fn function_name(param: Type) -> ReturnType {
    ...
}
```

---

## 4. 错误消息约定

### 4.1 默认消息

`expect()` 使用默认消息：

```rust
expect("unexpected failure in test")
```

### 4.2 中文消息

纯中文测试语境中允许使用中文消息：

```rust
expect("编译应成功")
expect("应该包含 color: red")
```

### 4.3 无消息 unwrap

禁止无消息的 `.unwrap()`。如需 unwrap，添加 `.expect()` 或 `.unwrap_or()` / `.unwrap_or_else()`。

---

## 5. 命名规范

### 5.1 文件命名

| 类别 | 格式 | 示例 |
|------|------|------|
| src 模块 | `snake_case.rs` | `selector_extend.rs` |
| src 测试辅助 | `snake_case.rs` | `hrx_support.rs` |
| 测试文件 | `<feature>_test.rs` | `compile_test.rs` |
| 集成测试 | `test_<name>.rs` | `reactor_test.rs` |

### 5.2 测试函数命名

| 前缀 | 用途 | 示例 |
|------|------|------|
| `test_` | 单元/集成测试 | `test_compile_simple` |
| `diag_` | sass-spec 诊断 | `diag_list` |
| `stats_` | 统计报告 | `stats_math` |
| `generate_` | 报告生成 | `cmd_stats` / `cmd_trend` (spec_store) |
| `check_` | 检测/验证 | `check_file_size_limits` |
| `hwb_` / `hsl_` | 颜色相关 | `hwb_degenerate_hue` |

### 5.3 变量/类型命名

遵循 Rust 惯例：
- 类型/枚举：`PascalCase`（`Selector`, `Combinator`）
- 函数/变量：`snake_case`（`extend_selector`, `match_pos`）
- 常量：`SCREAMING_SNAKE_CASE`（`MAX_DEPTH`）

---

## 6. 注释风格

### 6.1 行间注释

使用 `//` 单行注释，与代码同一行时保留一个空格：

```rust
let x = 42;  // 行尾注释
```

### 6.2 段落注释

使用 `//` 连续多行：

```rust
// 这里需要特别说明的是
// 多行注释应该这样写
```

### 6.3 Tracing 注释

`#[instrument]` 优先于手动 span 创建：

```rust
#[tracing::instrument(skip(large_param), fields(result = tracing::field::Empty))]
fn my_function(large_param: &BigType, input: &str) -> Result<...> {
    ...
}
```

---

## 7. Import 规范

### 7.1 分组顺序

```rust
// 1. std
use std::path::PathBuf;
use std::sync::Arc;

// 2. 外部 crate（如有）

// 3. 项目内部
use crate::css::selector_parser::parse_selector;
```

### 7.2 通配符限制

- `tests/` 文件允许 `use sasspile::*;` 简化导入
- `src/` 内部模块禁止 `use crate::*;`，使用精确导入

---

## 8. 行数限制

每个文件（源码和测试分别计算）不超过 **500 行**。超出必须拆分。

文件末尾检查：

```bash
wc -l src/eval/reactor.rs
```

---

## 9. 检查清单

修改文件后自检：

- [ ] Module header 存在且内容正确
- [ ] Section divider 使用 U+2500，对齐到 78 列
- [ ] 测试函数使用展开 `#[test]\nfn` 格式
- [ ] 错误消息统一或有合理本地化理由
- [ ] 文件名/函数名符合命名规范
- [ ] 文件行数 ≤ 500
