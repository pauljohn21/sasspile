# 项目文件拆分实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将 4 个超过 500 行的源文件按功能拆分，所有文件 ≤ 500 行，测试迁移到 `tests/` 目录。

**架构：** 利用 Rust 允许同一类型在多个文件中写多个 `impl` 块的特性，将 `Evaluator` 和 `Parser` 的方法按功能分散到不同文件。`call_builtin` 的巨型 match 拆分为子模块分派。

**技术栈：** Rust Edition 2024, toolchain 1.97

**设计文档：** `docs/superpowers/specs/2026-08-10-file-split-design.md`

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/parse/ast.rs` | 修改→缩小 | 只保留类型定义 |
| `src/parse/ast_impl.rs` | 新建 | Display + to_scss 实现 |
| `src/parse/mod.rs` | 修改→缩小 | Parser 结构 + 入口 |
| `src/parse/nodes.rs` | 新建 | 节点解析 + 参数解析 |
| `src/parse/at_rules.rs` | 新建 | @规则解析 |
| `src/parse/expr.rs` | 新建 | Pratt 表达式 + 数值/颜色解析 |
| `src/eval/mod.rs` | 修改→缩小 | Env + Evaluator 入口 |
| `src/eval/rule.rs` | 新建 | eval_rule + combine_selectors |
| `src/eval/value.rs` | 新建 | eval_value + binop + 算术 |
| `src/eval/control_flow.rs` | 新建 | eval_if/for/each/while |
| `src/eval/mixin.rs` | 新建 | eval_include + call_function |
| `src/eval/extend.rs` | 新建 | apply_extends |
| `src/eval/module.rs` | 新建 | resolve_file + load_module |
| `src/eval/color.rs` | 新建 | 颜色转换函数 |
| `src/eval/builtin.rs` | 新建 | call_builtin 分派入口 |
| `src/eval/builtin/*.rs` | 新建×7 | builtin 子模块 |
| `src/lex/mod.rs` | 修改→缩小 | 删除测试代码 |
| `tests/eval_test.rs` | 新建 | eval 测试 |
| `tests/parse_test.rs` | 新建 | parse 测试 |
| `tests/ast_test.rs` | 新建 | ast Display 测试 |
| `tests/to_scss_test.rs` | 新建 | to_scss 测试 |
| `tests/lex_test.rs` | 新建 | lex 测试 |
| `Cargo.toml` | 不变 | 无新依赖 |

---

## 任务 1：迁移测试代码到 tests/ 目录

**文件：**
- 创建：`tests/eval_test.rs`, `tests/parse_test.rs`, `tests/ast_test.rs`, `tests/to_scss_test.rs`, `tests/lex_test.rs`
- 修改：`src/eval/mod.rs`（删除 2114-2144 行的 `#[cfg(test)] mod tests`）
- 修改：`src/parse/mod.rs`（删除 1093-1148 行的 `#[cfg(test)] mod tests`）
- 修改：`src/parse/ast.rs`（删除 312-383 和 584-698 行的两个 `#[cfg(test)]` 模块）
- 修改：`src/lex/mod.rs`（删除 405-510 行的 `#[cfg(test)] mod tests`）

- [ ] **步骤 1：创建 tests/eval_test.rs**

从 `src/eval/mod.rs` 的 2114-2144 行提取测试代码：

```rust
use sasspile::compile_expanded;

#[test]
fn test_eval_simple() {
    let css = compile_expanded("a { color: red; }").unwrap();
    assert!(css.contains("a {"), "Missing a {:?}", css);
    assert!(css.contains("color: red;"), "Missing color: red; {:?}", css);
}

#[test]
fn test_eval_variable() {
    let css = compile_expanded("$x: blue; a { color: $x; }").unwrap();
    assert!(css.contains("color: blue"), "Missing color: blue {:?}", css);
}
```

- [ ] **步骤 2：创建 tests/parse_test.rs**

从 `src/parse/mod.rs` 的 1093-1148 行提取测试代码，添加必要的 import：

```rust
use sasspile::parse::{Parser, ast::*};
use sasspile::lex::Lexer;
use sasspile::lex::token::Token;

fn parse(input: &str) -> Ast {
    let tokens: Vec<Token> = Lexer::new(input)
        .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace) | Ok(Token::Eof)))
        .collect::<sasspile::error::Result<Vec<_>>>().unwrap();
    Parser::parse(&tokens)
}
```
然后粘贴原测试函数（test_parse_rule, test_parse_variable, test_parse_if, test_parse_mixin, test_parse_expr_precedence）。

- [ ] **步骤 3：创建 tests/ast_test.rs**

从 `src/parse/ast.rs` 的 312-383 行提取测试代码（test_number_display 到 test_color_rgba），添加 `use sasspile::parse::ast::*;`。

- [ ] **步骤 4：创建 tests/to_scss_test.rs**

从 `src/parse/ast.rs` 的 584-698 行提取 to_scss_tests 模块内容，添加 `use sasspile::parse::ast::*;`。

- [ ] **步骤 5：创建 tests/lex_test.rs**

从 `src/lex/mod.rs` 的 405-510 行提取测试代码，添加必要的 `use sasspile::lex::*;` 和 `use sasspile::lex::token::Token;`。

- [ ] **步骤 6：从源文件中删除所有 #[cfg(test)] 模块**

从 `src/eval/mod.rs`、`src/parse/mod.rs`、`src/parse/ast.rs`、`src/lex/mod.rs` 中删除所有 `#[cfg(test)] mod tests { ... }` 和 `#[cfg(test)] mod to_scss_tests { ... }` 块。

- [ ] **步骤 7：编译检查 + 运行测试**

运行：`cargo check && cargo test --lib 2>&1 | tail -3 && cargo test --test eval_test --test parse_test --test ast_test --test to_scss_test --test lex_test 2>&1 | tail -5`
预期：lib 测试全通过（数量减少，因为测试移走了），5 个新测试文件全通过。

- [ ] **步骤 8：Commit**

```bash
git add tests/eval_test.rs tests/parse_test.rs tests/ast_test.rs tests/to_scss_test.rs tests/lex_test.rs src/eval/mod.rs src/parse/mod.rs src/parse/ast.rs src/lex/mod.rs
git commit -m "refactor: 迁移测试代码到 tests/ 目录 — eval/parse/ast/lex"
```

---

## 任务 2：拆分 parse/ast.rs → ast.rs + ast_impl.rs

**文件：**
- 修改：`src/parse/ast.rs`（只保留 1-310 行：类型定义）
- 创建：`src/parse/ast_impl.rs`（原 235-570 行：Display + to_scss）
- 修改：`src/parse/mod.rs`（添加 `mod ast_impl;`）

- [ ] **步骤 1：创建 src/parse/ast_impl.rs**

将 `src/parse/ast.rs` 中 `impl std::fmt::Display for Value` 和 `impl Node { pub fn to_scss(...) }` 的全部代码移动到 `src/parse/ast_impl.rs`。

文件头部添加：
```rust
use super::ast::*;
use crate::css::node::Color;
```

- [ ] **步骤 2：从 ast.rs 中删除已移动的代码**

从 `src/parse/ast.rs` 中删除 Display impl 和 to_scss impl 块。文件只保留类型定义。

- [ ] **步骤 3：在 parse/mod.rs 中声明子模块**

在 `src/parse/mod.rs` 中，确保有：
```rust
pub mod ast;
mod ast_impl;
```

- [ ] **步骤 4：编译检查 + 测试**

运行：`cargo check && cargo test --lib 2>&1 | tail -3 && cargo test --test ast_test --test to_scss_test 2>&1 | tail -3`
预期：全通过。

- [ ] **步骤 5：Commit**

```bash
git add src/parse/ast.rs src/parse/ast_impl.rs src/parse/mod.rs
git commit -m "refactor: 拆分 ast.rs → ast.rs(类型) + ast_impl.rs(Display+to_scss)"
```

---

## 任务 3：拆分 parse/mod.rs → mod.rs + nodes.rs + at_rules.rs + expr.rs

**文件：**
- 修改：`src/parse/mod.rs`（只保留 1-69 行 + mod 声明）
- 创建：`src/parse/nodes.rs`（原 70-248 + 601-725 行）
- 创建：`src/parse/at_rules.rs`（原 249-600 行）
- 创建：`src/parse/expr.rs`（原 757-1084 行 + 自由函数）

- [ ] **步骤 1：创建 src/parse/nodes.rs**

将 `src/parse/mod.rs` 中 70-248 行（`parse_node` 到 `parse_body`）和 601-725 行（`parse_params` 到 `expect_keyword`）移动到 `src/parse/nodes.rs`。

文件头部添加：
```rust
use super::ast::*;
use super::Parser;
use crate::error::Result;
use crate::lex::token::Token;
```

代码包裹在 `impl<'tok> Parser<'tok> { ... }` 中。

- [ ] **步骤 2：创建 src/parse/at_rules.rs**

将 `src/parse/mod.rs` 中 249-600 行（`parse_at_rule` 到 `parse_at_params`）移动到 `src/parse/at_rules.rs`。

文件头部添加：
```rust
use super::ast::*;
use super::Parser;
use crate::error::Result;
```

代码包裹在 `impl<'tok> Parser<'tok> { ... }` 中。

- [ ] **步骤 3：创建 src/parse/expr.rs**

将 `src/parse/mod.rs` 中 757-1084 行（`parse_expr` 到 `hex1`）移动到 `src/parse/expr.rs`。

注意：`parse_expr`、`is_value_start`、`parse_prefix`、`peek_binding_power` 是 `Parser` 方法，包裹在 `impl<'tok> Parser<'tok> { ... }` 中。`parse_number`、`parse_hash_color`、`hex2`、`hex1` 是自由函数，放在 `impl` 块之外。

文件头部添加：
```rust
use super::ast::*;
use super::Parser;
use crate::error::Result;
use crate::lex::token::Token;
```

- [ ] **步骤 4：更新 parse/mod.rs**

`src/parse/mod.rs` 只保留 1-69 行（mod 声明 + import + Parser 结构 + parse() 入口 + 基础操作），并在末尾添加子模块声明：

```rust
pub mod ast;
mod ast_impl;
mod nodes;
mod at_rules;
mod expr;
```

- [ ] **步骤 5：编译检查 + 测试**

运行：`cargo check && cargo test --lib 2>&1 | tail -3 && cargo test --test parse_test 2>&1 | tail -3`
预期：全通过。

- [ ] **步骤 6：Commit**

```bash
git add src/parse/mod.rs src/parse/nodes.rs src/parse/at_rules.rs src/parse/expr.rs
git commit -m "refactor: 拆分 parse/mod.rs → mod.rs + nodes.rs + at_rules.rs + expr.rs"
```

---

## 任务 4：拆分 eval/mod.rs — 核心模块

**文件：**
- 修改：`src/eval/mod.rs`（只保留 1-310 行 + mod 声明）
- 创建：`src/eval/rule.rs`, `src/eval/value.rs`, `src/eval/control_flow.rs`, `src/eval/mixin.rs`, `src/eval/extend.rs`, `src/eval/module.rs`, `src/eval/color.rs`

- [ ] **步骤 1：创建 src/eval/rule.rs**

将 `src/eval/mod.rs` 中 `eval_rule`（312-400）、`combine_selectors`（403-421）、`eval_at_root`（1209-1213）移动到 `src/eval/rule.rs`。

文件头部：
```rust
use super::*;
use crate::css::node::CssNode;
use crate::error::Result;
use crate::parse::ast::*;
```

代码包裹在 `impl Evaluator { ... }` 中。

- [ ] **步骤 2：创建 src/eval/value.rs**

将 `eval_variable`（422-430）、`eval_value`（432-498）、`eval_binop`（499-527）、`add`（531-563）、`sub`（564-580）、`mul`（581-589）、`div`（590-603）、`modulo`（604-615）、`compare`（616-632）、`units_compatible`（633-652）、`inspect_value`（653-693）、`values_eq`（694-717）、`eval_interp_str`（718-748）、`eval_simple_expr`（750-768）移动到 `src/eval/value.rs`。

文件头部：
```rust
use super::*;
use crate::error::Result;
use crate::parse::ast::*;
use crate::parse::ast::BinOpKind;
```

- [ ] **步骤 3：创建 src/eval/control_flow.rs**

将 `eval_if`（770-783）、`eval_for`（784-812）、`eval_each`（813-851）、`eval_while`（852-872）移动到 `src/eval/control_flow.rs`。

文件头部：
```rust
use super::*;
use crate::css::node::CssNode;
use crate::error::Result;
use crate::parse::ast::*;
```

- [ ] **步骤 4：创建 src/eval/mixin.rs**

将 `eval_include`（875-891）、`bind_params`（892-938）、`call_function`（939-954）、`call_user_function`（1183-1207）、`eval_at_rule`（1216-1228）移动到 `src/eval/mixin.rs`。

文件头部：
```rust
use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::*;
```

- [ ] **步骤 5：创建 src/eval/extend.rs**

将 `apply_extends`（955-1020）移动到 `src/eval/extend.rs`。

文件头部：
```rust
use super::*;
use crate::css::node::CssNode;
```

- [ ] **步骤 6：创建 src/eval/module.rs**

将 `resolve_file`（1022-1057）、`load_module`（1058-1087）、`call_module_function`（1088-1182）移动到 `src/eval/module.rs`。

文件头部：
```rust
use super::*;
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use std::path::{Path, PathBuf};
```

- [ ] **步骤 7：创建 src/eval/color.rs**

将 `hsl_to_rgb`（1878-1916）、`hwb_to_rgb`（1917-1950）、`rgb_to_hsl`（1951-1986）、`simple_random`（1987-1994）、`builtin_rgba`（1996-2018）、`builtin_darken`（2020-2048）、`builtin_lighten`（2049-2077）、`builtin_mix`（2078-2115）移动到 `src/eval/color.rs`。

文件头部：
```rust
use super::*;
use crate::css::node::Color;
use crate::error::Result;
use crate::parse::ast::*;
```

- [ ] **步骤 8：更新 eval/mod.rs**

`src/eval/mod.rs` 只保留 1-310 行（Env + ModuleExports + MixinDef + FunctionDef + Evaluator + evaluate + eval_nodes + eval_node），并在末尾添加子模块声明：

```rust
mod rule;
mod value;
mod control_flow;
mod mixin;
mod extend;
mod module;
mod color;
mod builtin;
```

注意：`eval/mod.rs` 中的 `Evaluator` 需要 `pub(crate)` 标记，让子模块能访问。`Env` 的字段也需要对子模块可见——可以通过 `pub(crate)` 或者将子模块的 `use super::*` 来实现（子模块在同一 crate 内可以访问父模块的私有项，只要通过 `use super::*` 导入）。

- [ ] **步骤 9：编译检查 + 测试**

运行：`cargo check 2>&1 | head -30`
如果有 import 错误，修复 import 路径。

运行：`cargo test --lib 2>&1 | tail -3 && cargo test --test eval_test 2>&1 | tail -3`
预期：全通过。

- [ ] **步骤 10：Commit**

```bash
git add src/eval/mod.rs src/eval/rule.rs src/eval/value.rs src/eval/control_flow.rs src/eval/mixin.rs src/eval/extend.rs src/eval/module.rs src/eval/color.rs
git commit -m "refactor: 拆分 eval/mod.rs 核心模块 — rule/value/control_flow/mixin/extend/module/color"
```

---

## 任务 5：拆分 call_builtin → builtin.rs + 7 个子模块

**文件：**
- 创建：`src/eval/builtin.rs`（分派入口）
- 创建：`src/eval/builtin/math.rs`, `string.rs`, `color.rs`, `list.rs`, `map.rs`, `meta.rs`, `selector.rs`
- 修改：`src/eval/mod.rs`（确认 `mod builtin;` 声明）

- [ ] **步骤 1：创建 src/eval/builtin/math.rs**

提取 `call_builtin` 中 math 相关的 match arm（abs/ceil/floor/round/min/max/percentage + math.* 模块函数映射）：

```rust
use crate::error::{Result, SassError};
use crate::parse::ast::*;

pub fn call(name: &str, args: &[Value]) -> Result<Option<Value>> {
    match name {
        "abs" => match args {
            [Value::Number(n, u)] => Ok(Some(Value::Number(n.abs(), u.clone()))),
            _ => Err(SassError::Eval("abs 需要 1 个数字参数".into())),
        },
        "ceil" => match args { ... },
        "floor" => match args { ... },
        // ... 所有 math 函数 ...
        _ => Ok(None),  // 不匹配，交给下一个子模块
    }
}
```

- [ ] **步骤 2：创建 src/eval/builtin/string.rs**

提取 string 相关 arm（str-length/to-upper-case/to-lower-case/unquote/quote/str-index/str-slice/str-insert/str-split/unique-id）。同样的 `pub fn call(name, args) -> Result<Option<Value>>` 签名。

- [ ] **步骤 3：创建 src/eval/builtin/color.rs**

提取 color 相关 arm（rgb/rgba/darken/lighten/mix/invert/grayscale/complement/hwb/hsl/hsla/adjust-hue/saturate/desaturate/transparentize/fade-out/opacify/fade-in/alpha/opacity/red/green/blue/hue/saturation/lightness/color-channel/adjust-color/change-color/scale-color）。

注意：`builtin_rgba`、`builtin_darken`、`builtin_lighten`、`builtin_mix` 已经是独立函数（在 `eval/color.rs` 中），这里只做分派调用。

- [ ] **步骤 4：创建 src/eval/builtin/list.rs**

提取 list 相关 arm（length/nth/append/join/index/separator/set-nth/zip/is-bracketed/list-slash）。

- [ ] **步骤 5：创建 src/eval/builtin/map.rs**

提取 map 相关 arm（map-get/map-merge/map-remove/map-keys/map-values/map-has-key/map-deep-remove/map-set）。

- [ ] **步骤 6：创建 src/eval/builtin/meta.rs**

提取 meta 相关 arm（type-of/inspect/keywords/get-function/call/mixin-exists/function-exists/global-variable-exists/variable-exists）。

- [ ] **步骤 7：创建 src/eval/builtin/selector.rs**

提取 selector 相关 arm（selector-append/selector-nest/selector-is-super/selector-parse/selector-simple-selectors/selector-unify/selector-extend）。

- [ ] **步骤 8：创建 src/eval/builtin.rs（分派入口）**

```rust
pub mod math;
pub mod string;
pub mod color;
pub mod list;
pub mod map;
pub mod meta;
pub mod selector;

use crate::error::{Result, SassError};
use crate::parse::ast::*;

/// 内建函数分派——按类别依次尝试子模块。
pub fn call_builtin(name: &str, args: &[Value], env: &super::Env) -> Result<Value> {
    // 先尝试 math
    if let Some(v) = math::call(name, args)? { return Ok(v); }
    // 再尝试 string
    if let Some(v) = string::call(name, args)? { return Ok(v); }
    // 再尝试 color
    if let Some(v) = color::call(name, args)? { return Ok(v); }
    // 再尝试 list
    if let Some(v) = list::call(name, args)? { return Ok(v); }
    // 再尝试 map
    if let Some(v) = map::call(name, args)? { return Ok(v); }
    // 再尝试 meta
    if let Some(v) = meta::call(name, args)? { return Ok(v); }
    // 再尝试 selector
    if let Some(v) = selector::call(name, args)? { return Ok(v); }
    // 不匹配任何子模块
    Err(SassError::UndefinedFunction(name.to_string()))
}
```

注意：部分函数需要 `env` 参数（如 `variable-exists`、`mixin-exists` 等）。这些函数的 `call` 签名需要额外接收 `env`：

```rust
pub fn call(name: &str, args: &[Value], env: &Env) -> Result<Option<Value>>
```

所有子模块统一此签名。不需要 `env` 的函数忽略该参数。

- [ ] **步骤 9：从 eval/mod.rs 中删除 call_builtin**

确认 `call_builtin` 函数已从 `src/eval/mod.rs` 中删除（它在任务 4 中保留在 mod.rs 里，现在移到 `builtin.rs`）。

确认 `src/eval/mod.rs` 中有 `mod builtin;` 声明。

- [ ] **步骤 10：编译检查 + 测试**

运行：`cargo check 2>&1 | head -30`
如果有错误，修复 import 和可见性。

运行：`cargo test --lib 2>&1 | tail -3 && cargo test --test eval_test 2>&1 | tail -3`
预期：全通过。

- [ ] **步骤 11：Commit**

```bash
git add src/eval/builtin.rs src/eval/builtin/ src/eval/mod.rs
git commit -m "refactor: 拆分 call_builtin → builtin.rs + 7 个子模块"
```

---

## 任务 6：最终验证 + 更新文档

**文件：**
- 不修改代码
- 更新：`AGENTS.md`（如有文件行数引用变化）

- [ ] **步骤 1：验证所有文件 ≤ 500 行**

运行：`find src -name '*.rs' -exec wc -l {} + | sort -rn | head -20`
预期：所有文件 ≤ 500 行。

- [ ] **步骤 2：全量测试回归**

运行：`cargo test --lib 2>&1 | tail -3 && cargo test --test eval_test --test parse_test --test ast_test --test to_scss_test --test lex_test --test common_test 2>&1 | tail -5`
预期：全部通过。

- [ ] **步骤 3：Commit**

```bash
git add -A
git commit -m "refactor: 文件拆分完成 — 全部文件 ≤ 500 行"
```

---

## 自检

### 1. 规格覆盖度

| 设计文档章节 | 实现任务 | 覆盖 |
|-------------|---------|------|
| §2 eval/ 拆分核心模块 | 任务 4 | ✅ |
| §2.2 Builtin 子模块 | 任务 5 | ✅ |
| §2.3 颜色转换函数 | 任务 4 步骤 7 | ✅ |
| §3 parse/ 拆分 | 任务 3 | ✅ |
| §4 ast.rs 拆分 | 任务 2 | ✅ |
| §5 lex/mod.rs | 任务 1（测试迁移） | ✅ |
| §6 测试迁移 | 任务 1 | ✅ |

### 2. 占位符扫描

- 无 TODO/待定 ✅
- builtin 子模块的具体 match arm 用 `...` 省略表示（因为完整代码是从原文件移动，不需要重写）——这不是占位符，是"从原文件移动"的指示 ✅

### 3. 类型一致性

- `pub fn call(name: &str, args: &[Value], env: &Env) -> Result<Option<Value>>` — 所有 builtin 子模块统一签名 ✅
- `impl Evaluator` 和 `impl Parser` — 在多个文件中分散，Rust 允许 ✅
- `use super::*` — 子模块通过此导入访问父模块的类型和方法 ✅
