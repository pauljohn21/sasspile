# 调试工具链实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 为 sasspile 添加 CSS diff 工具、sass-spec 最小化工具、color/selector 值快照 tracing events，并升级 init_tracing 支持 target 过滤。

**架构：** 在现有 tracing Span 架构上叠加 Event 值快照层；新增两个测试辅助模块（CSS diff + 最小化）；init_tracing 从 `.with_target(false)` 改为 `.with_target(true)`。全部基于现有 `tracing` + `tracing-subscriber` 依赖，零新依赖。

**技术栈：** Rust Edition 2024, toolchain 1.97, tracing 0.1, tracing-subscriber 0.3

**设计文档：** `docs/superpowers/specs/2026-08-10-tracing-debug-events-design.md`

---

## 文件结构

| 文件 | 操作 | 职责 |
|------|------|------|
| `src/lib.rs` | 修改 | `init_tracing` 升级：显示 target、level、彩色 |
| `src/parse/ast.rs` | 修改 | 新增 `Node::to_scss()` 方法：AST → SCSS 序列化 |
| `tests/common/mod.rs` | 新建 | 共享 CSS diff 模块：`diff_css()` + tracing events |
| `tests/cf_diag.rs` | 修改 | 集成 `diff_css` 替换只比较第一行的逻辑 |
| `tests/minimize.rs` | 新建 | Delta debugging 最小化工具 + tracing events |
| `src/eval/mod.rs` | 修改 | color/extend/binop 值快照 tracing events |
| `Cargo.toml` | 不变 | 无新依赖 |

---

## 任务 1：init_tracing 升级

**文件：**
- 修改：`src/lib.rs:38-45`

- [ ] **步骤 1：编写失败的测试**

在 `src/lib.rs` 的 `mod tests` 中添加测试：

```rust
#[test]
fn test_init_tracing_shows_target() {
    // init_tracing 应该启用 target 显示
    // 验证：不 panic 且能正常初始化
    init_tracing();
    tracing::info!(target: "test_target_check", "tracing target visibility test");
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test --lib test_init_tracing_shows_target -- --nocapture`
预期：PASS（测试本身不会失败，但用来验证修改不影响现有行为）
注意：此步骤作为基线——修改 init_tracing 后再运行确认仍 PASS。

- [ ] **步骤 3：编写实现代码**

将 `src/lib.rs` 中的 `init_tracing` 替换为：

```rust
/// 初始化 tracing 日志——用 `RUST_LOG` 环境变量控制级别和 target。
///
/// # Target 过滤
///
/// ```bash
/// # 只看颜色相关 events
/// RUST_LOG="sasspile::color=debug" cargo test -- --nocapture
///
/// # 组合多个 target
/// RUST_LOG="sasspile::color=trace,sasspile::extend=info" cargo test -- --nocapture
/// ```
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

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test --lib test_init_tracing_shows_target -- --nocapture`
预期：PASS，输出中包含 `test_target_check`

运行：`cargo test --lib 2>&1 | tail -3`
预期：65 个测试全通过

- [ ] **步骤 5：Commit**

```bash
git add src/lib.rs
git commit -m "feat: init_tracing 升级 — 显示 target/level，支持 per-target 过滤"
```

---

## 任务 2：Node::to_scss() — 基础节点

**文件：**
- 修改：`src/parse/ast.rs`（在文件末尾，`impl std::fmt::Display for Value` 之后）
- 测试：`src/parse/ast.rs` 中的 `#[cfg(test)] mod tests`

- [ ] **步骤 1：编写失败的测试**

在 `src/parse/ast.rs` 末尾添加测试模块：

```rust
#[cfg(test)]
mod to_scss_tests {
    use super::*;

    #[test]
    fn test_rule_to_scss() {
        let node = Node::Rule {
            selector: "a".into(),
            body: vec![Node::Decl {
                property: "color".into(),
                value: Value::String("red".into(), false),
                important: false,
            }],
        };
        let scss = node.to_scss(0);
        assert!(scss.contains("a {"));
        assert!(scss.contains("color: red;"));
        assert!(scss.contains("}"));
    }

    #[test]
    fn test_decl_to_scss() {
        let node = Node::Decl {
            property: "width".into(),
            value: Value::Number(100.0, Some("px".into())),
            important: true,
        };
        let scss = node.to_scss(0);
        assert_eq!(scss, "width: 100px !important;");
    }

    #[test]
    fn test_variable_to_scss() {
        let node = Node::Variable {
            name: "color".into(),
            value: Value::String("blue".into(), false),
            flags: VarFlags { default: true, global: false },
        };
        let scss = node.to_scss(0);
        assert_eq!(scss, "$color: blue !default;");
    }

    #[test]
    fn test_comment_to_scss() {
        let silent = Node::Comment("hello".into(), true);
        let loud = Node::Comment("world".into(), false);
        assert_eq!(silent.to_scss(0), "// hello");
        assert_eq!(loud.to_scss(0), "/* world */");
    }
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test --lib to_scss_tests -- --nocapture`
预期：编译失败，`to_scss` 方法不存在

- [ ] **步骤 3：编写实现代码**

在 `src/parse/ast.rs` 中，`impl std::fmt::Display for Value` 之后添加：

```rust
impl Node {
    /// 将 AST 节点序列化回 SCSS 源码——用于最小化工具。
    pub fn to_scss(&self, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        match self {
            Node::Rule { selector, body } => {
                let body: String = body.iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                if body.is_empty() {
                    format!("{pad}{selector} {{}}")
                } else {
                    format!("{pad}{selector} {{\n{body}\n{pad}}}")
                }
            }
            Node::Decl { property, value, important } => {
                let imp = if *important { " !important" } else { "" };
                format!("{pad}{property}: {value}{imp};")
            }
            Node::Variable { name, value, flags } => {
                let mut s = format!("{pad}${name}: {value}");
                if flags.default { s.push_str(" !default"); }
                if flags.global { s.push_str(" !global"); }
                s.push(';');
                s
            }
            Node::Comment(text, silent) => {
                if *silent { format!("{pad}// {text}") }
                else { format!("{pad}/* {text} */") }
            }
            // —— 以下变体在任务 3 实现 ——
            _ => format!("{pad}/* TODO: to_scss for this variant */"),
        }
    }
}
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test --lib to_scss_tests -- --nocapture`
预期：4 个测试 PASS

- [ ] **步骤 5：Commit**

```bash
git add src/parse/ast.rs
git commit -m "feat: Node::to_scss() 基础节点序列化 — Rule/Decl/Variable/Comment"
```

---

## 任务 3：Node::to_scss() — 全部剩余节点

**文件：**
- 修改：`src/parse/ast.rs`（替换任务 2 中的 `_ =>` 占位分支）

- [ ] **步骤 1：编写失败的测试**

在 `to_scss_tests` 模块中追加：

```rust
    #[test]
    fn test_if_to_scss() {
        let node = Node::If {
            branches: vec![(Value::Bool(true), vec![Node::Decl {
                property: "color".into(), value: Value::String("red".into(), false), important: false,
            }])],
            else_body: Some(vec![Node::Decl {
                property: "color".into(), value: Value::String("blue".into(), false), important: false,
            }]),
        };
        let scss = node.to_scss(0);
        assert!(scss.contains("@if true"));
        assert!(scss.contains("@else"));
    }

    #[test]
    fn test_for_to_scss() {
        let node = Node::For {
            var: "i".into(),
            from: Value::Number(1.0, None),
            to: Value::Number(10.0, None),
            inclusive: true,
            body: vec![Node::Decl {
                property: "w".into(), value: Value::Variable("i".into()), important: false,
            }],
        };
        let scss = node.to_scss(0);
        assert!(scss.contains("@for $i from 1 through 10"));
    }

    #[test]
    fn test_include_to_scss() {
        let node = Node::Include {
            name: "my-mixin".into(),
            args: vec![],
            content: None,
        };
        assert_eq!(node.to_scss(0), "@include my-mixin;");
    }

    #[test]
    fn test_extend_to_scss() {
        let node = Node::Extend { selector: ".btn".into(), optional: true };
        assert_eq!(node.to_scss(0), "@extend .btn !optional;");
    }

    #[test]
    fn test_use_to_scss() {
        let node = Node::Use {
            url: "sass:color".into(), namespace: None, star: false, config: vec![],
        };
        assert_eq!(node.to_scss(0), "@use \"sass:color\";");
    }
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test --lib to_scss_tests -- --nocapture`
预期：新增 5 个测试 FAIL（输出 `/* TODO: to_scss for this variant */`）

- [ ] **步骤 3：编写实现代码**

将任务 2 中的 `_ => format!(...)` 占位分支替换为所有剩余变体的实现：

```rust
            // —— 控制流 ——
            Node::If { branches, else_body } => {
                let mut s = String::new();
                for (i, (cond, body)) in branches.iter().enumerate() {
                    let kw = if i == 0 { "@if" } else { "@else if" };
                    let body_s: String = body.iter()
                        .map(|n| n.to_scss(indent + 1))
                        .collect::<Vec<_>>()
                        .join("\n");
                    s.push_str(&format!("{pad}{kw} {cond} {{\n{body_s}\n{pad}}}"));
                    if i < branches.len() - 1 || else_body.is_some() { s.push('\n'); }
                }
                if let Some(eb) = else_body {
                    let body_s: String = eb.iter()
                        .map(|n| n.to_scss(indent + 1))
                        .collect::<Vec<_>>()
                        .join("\n");
                    s.push_str(&format!("{pad}@else {{\n{body_s}\n{pad}}}"));
                }
                s
            }
            Node::For { var, from, to, inclusive, body } => {
                let kw = if *inclusive { "through" } else { "to" };
                let body_s: String = body.iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@for ${var} from {from} {kw} {to} {{\n{body_s}\n{pad}}}")
            }
            Node::Each { vars, list, body } => {
                let vars_s = vars.iter().map(|v| format!("${v}")).collect::<Vec<_>>().join(", ");
                let body_s: String = body.iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@each {vars_s} in {list} {{\n{body_s}\n{pad}}}")
            }
            Node::While { cond, body } => {
                let body_s: String = body.iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@while {cond} {{\n{body_s}\n{pad}}}")
            }
            // —— Mixin / Function ——
            Node::MixinDef { name, params, body } => {
                let params_s = params.iter().map(|p| {
                    let s = format!("${}", p.name);
                    if p.rest { format!("{s}...") }
                    else if let Some(d) = &p.default { format!("{s}: {d}") }
                    else { s }
                }).collect::<Vec<_>>().join(", ");
                let body_s: String = body.iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@mixin {name}({params_s}) {{\n{body_s}\n{pad}}}")
            }
            Node::Include { name, args, content } => {
                let args_s = args.iter().map(|a| {
                    if let Some(n) = &a.name { format!("${n}: ") } else { String::new() }
                        + &a.value.to_string()
                        + if a.spread { "..." } else { "" }
                }).collect::<Vec<_>>().join(", ");
                let base = if args_s.is_empty() {
                    format!("{pad}@include {name};")
                } else {
                    format!("{pad}@include {name}({args_s});")
                };
                if let Some(content) = content {
                    let content_s: String = content.iter()
                        .map(|n| n.to_scss(indent + 1))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{base}\n{pad}{{\n{content_s}\n{pad}}}")
                } else {
                    base
                }
            }
            Node::Content => format!("{pad}@content;"),
            Node::FunctionDef { name, params, body } => {
                let params_s = params.iter().map(|p| {
                    let s = format!("${}", p.name);
                    if p.rest { format!("{s}...") }
                    else if let Some(d) = &p.default { format!("{s}: {d}") }
                    else { s }
                }).collect::<Vec<_>>().join(", ");
                let body_s: String = body.iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@function {name}({params_s}) {{\n{body_s}\n{pad}}}")
            }
            Node::Return(v) => format!("{pad}@return {v};"),
            // —— 模块系统 ——
            Node::Use { url, namespace, star, config } => {
                let mut s = format!("{pad}@use \"{url}\"");
                if *star { s.push_str(" as *"); }
                else if let Some(ns) = namespace { s.push_str(&format!(" as {ns}")); }
                if !config.is_empty() {
                    let cfg: String = config.iter()
                        .map(|(k, v)| format!("${k}: {v}"))
                        .collect::<Vec<_>>().join(", ");
                    s.push_str(&format!(" with ({cfg})"));
                }
                s.push(';');
                s
            }
            Node::Forward { url, show, hide, prefix } => {
                let mut s = format!("{pad}@forward \"{url}\"");
                if let Some(p) = prefix { s.push_str(&format!(" as {p}-*")); }
                if !show.is_empty() {
                    s.push_str(&format!(" show {}", show.join(", ")));
                }
                if !hide.is_empty() {
                    s.push_str(&format!(" hide {}", hide.join(", ")));
                }
                s.push(';');
                s
            }
            Node::Import { url } => format!("{pad}@import \"{url}\";"),
            // —— 其他指令 ——
            Node::Extend { selector, optional } => {
                let opt = if *optional { " !optional" } else { "" };
                format!("{pad}@extend {selector}{opt};")
            }
            Node::AtRoot { query, body } => {
                let body_s: String = body.iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Some(q) = query {
                    format!("{pad}@at-root {q} {{\n{body_s}\n{pad}}}")
                } else {
                    format!("{pad}@at-root {{\n{body_s}\n{pad}}}")
                }
            }
            Node::AtRule { name, params, body } => {
                let params_s = params.as_deref().unwrap_or("");
                match body {
                    Some(nodes) => {
                        let body_s: String = nodes.iter()
                            .map(|n| n.to_scss(indent + 1))
                            .collect::<Vec<_>>()
                            .join("\n");
                        if params_s.is_empty() {
                            format!("{pad}@{name} {{\n{body_s}\n{pad}}}")
                        } else {
                            format!("{pad}@{name} {params_s} {{\n{body_s}\n{pad}}}")
                        }
                    }
                    None => {
                        if params_s.is_empty() { format!("{pad}@{name};") }
                        else { format!("{pad}@{name} {params_s};") }
                    }
                }
            }
            Node::Warn(v) => format!("{pad}@warn {v};"),
            Node::Debug(v) => format!("{pad}@debug {v};"),
            Node::Error(v) => format!("{pad}@error {v};"),
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test --lib to_scss_tests -- --nocapture`
预期：9 个测试全 PASS

运行：`cargo test --lib 2>&1 | tail -3`
预期：65 + 9 = 74 测试全通过

- [ ] **步骤 5：Commit**

```bash
git add src/parse/ast.rs
git commit -m "feat: Node::to_scss() 全部节点序列化 — 控制流/Mixin/Function/模块/指令"
```

---

## 任务 4：CSS Diff 模块

**文件：**
- 创建：`tests/common/mod.rs`
- 测试：同文件内的 `#[cfg(test)]` 模块

- [ ] **步骤 1：编写失败的测试**

创建 `tests/common/mod.rs`：

```rust
//! 共享测试工具——CSS diff 和辅助函数。
//!
//! 被 `cf_diag.rs` 和 `minimize.rs` 引用。

/// CSS diff 行类型。
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLine {
    Changed { line: usize, expected: String, actual: String },
    ExtraExpected { line: usize, content: String },
    ExtraActual { line: usize, content: String },
}

/// CSS diff 结果。
#[derive(Debug, Clone, Default)]
pub struct DiffResult {
    pub lines: Vec<DiffLine>,
}

impl DiffResult {
    /// 分类错误模式——用于统计。
    pub fn classify(&self) -> &'static str {
        if self.lines.is_empty() { return "identical"; }
        let has_missing = self.lines.iter().any(|l| matches!(l, DiffLine::ExtraExpected { .. }));
        let has_extra = self.lines.iter().any(|l| matches!(l, DiffLine::ExtraActual { .. }));
        if has_missing && !has_extra { "missing_output" }
        else if has_extra && !has_missing { "extra_output" }
        else { "content_diff" }
    }

    /// 格式化为终端可读文本。
    pub fn format_terminal(&self) -> String {
        self.lines.iter().map(|l| match l {
            DiffLine::Changed { line, expected, actual } =>
                format!("  L{line}: exp='{expected}' act='{actual}'"),
            DiffLine::ExtraExpected { line, content } =>
                format!("  L{line}: exp='{content}' act=(missing)"),
            DiffLine::ExtraActual { line, content } =>
                format!("  L{line}: exp=(missing) act='{content}'"),
        }).collect::<Vec<_>>().join("\n")
    }
}

/// CSS 逐行 diff——返回结构化差异结果。
///
/// 发出 tracing events：
/// - `cssdiff` info: diff 检测摘要
/// - `cssdiff` debug: 每行差异详情
pub fn diff_css(expected: &str, actual: &str) -> DiffResult {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    tracing::info!(
        target: "cssdiff",
        n_expected = expected_lines.len(),
        n_actual = actual_lines.len(),
        "diff detected"
    );

    let mut result = DiffResult::default();
    let max = expected_lines.len().max(actual_lines.len());

    for i in 0..max {
        let e = expected_lines.get(i).copied();
        let a = actual_lines.get(i).copied();
        match (e, a) {
            (Some(e), Some(a)) if e == a => continue,
            (Some(e), Some(a)) => {
                tracing::debug!(
                    target: "cssdiff",
                    line = i + 1,
                    expected = e,
                    actual = a,
                    "line diff"
                );
                result.lines.push(DiffLine::Changed {
                    line: i + 1,
                    expected: e.to_string(),
                    actual: a.to_string(),
                });
            }
            (Some(e), None) => {
                tracing::debug!(
                    target: "cssdiff",
                    line = i + 1,
                    expected = e,
                    actual = "(missing)",
                    "line missing in actual"
                );
                result.lines.push(DiffLine::ExtraExpected {
                    line: i + 1,
                    content: e.to_string(),
                });
            }
            (None, Some(a)) => {
                tracing::debug!(
                    target: "cssdiff",
                    line = i + 1,
                    expected = "(missing)",
                    actual = a,
                    "extra line in actual"
                );
                result.lines.push(DiffLine::ExtraActual {
                    line: i + 1,
                    content: a.to_string(),
                });
            }
            (None, None) => break,
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identical() {
        let result = diff_css("a { color: red; }", "a { color: red; }");
        assert_eq!(result.lines.len(), 0);
        assert_eq!(result.classify(), "identical");
    }

    #[test]
    fn test_diff_changed_line() {
        let result = diff_css("a { color: red; }", "a { color: blue; }");
        assert_eq!(result.lines.len(), 1);
        assert!(matches!(&result.lines[0], DiffLine::Changed { line: 1, expected, actual }
            if expected == "a { color: red; }" && actual == "a { color: blue; }"));
        assert_eq!(result.classify(), "content_diff");
    }

    #[test]
    fn test_diff_missing_output() {
        let result = diff_css("a { color: red; }\nb { color: blue; }", "a { color: red; }");
        assert_eq!(result.lines.len(), 1);
        assert!(matches!(&result.lines[0], DiffLine::ExtraExpected { line: 2, .. }));
        assert_eq!(result.classify(), "missing_output");
    }

    #[test]
    fn test_diff_extra_output() {
        let result = diff_css("a { color: red; }", "a { color: red; }\nb { color: blue; }");
        assert_eq!(result.lines.len(), 1);
        assert!(matches!(&result.lines[0], DiffLine::ExtraActual { line: 2, .. }));
        assert_eq!(result.classify(), "extra_output");
    }

    #[test]
    fn test_format_terminal() {
        let result = diff_css("a { color: red; }", "a { color: blue; }");
        let formatted = result.format_terminal();
        assert!(formatted.contains("L1"));
        assert!(formatted.contains("red"));
        assert!(formatted.contains("blue"));
    }
}
```

- [ ] **步骤 2：运行测试验证失败**

运行：`cargo test --test common -- --nocapture`
预期：编译错误（`tests/common/mod.rs` 不是独立的测试目标）

注意：`tests/common/mod.rs` 不能独立运行。需要在 `cf_diag.rs` 或 `minimize.rs` 中 `mod common;` 引用。先创建一个临时测试文件 `tests/common_test.rs`：

```rust
mod common;
```

然后运行：`cargo test --test common_test -- --nocapture`
预期：5 个测试 PASS（diff 函数已实现，测试应直接通过）

或者，如果测试直接 PASS，说明实现正确。此任务的 TDD 顺序是：先写测试和实现在一起（因为它们在同一文件），验证编译和测试通过。

- [ ] **步骤 3：创建临时测试入口**

创建 `tests/common_test.rs`：

```rust
mod common;
```

- [ ] **步骤 4：运行测试验证通过**

运行：`cargo test --test common_test -- --nocapture`
预期：5 个测试 PASS

- [ ] **步骤 5：Commit**

```bash
git add tests/common/mod.rs tests/common_test.rs
git commit -m "feat: CSS diff 模块 — 逐行对比 + tracing events + 分类统计"
```

---

## 任务 5：集成 CSS Diff 到 cf_diag

**文件：**
- 修改：`tests/cf_diag.rs:50-103`（`diag` 函数）

- [ ] **步骤 1：添加 mod common 声明**

在 `tests/cf_diag.rs` 顶部添加：

```rust
mod common;
use common::diff_css;
```

- [ ] **步骤 2：修改 diag 函数中的 diff 逻辑**

将 `diag` 函数中 `Ok(actual) => { ... }` 分支替换为：

```rust
                    Ok(actual) => {
                        if actual.trim() != expected.trim() {
                            shown += 1;
                            let diff = diff_css(expected.trim(), actual.trim());
                            let key = diff.classify();
                            *err_types.entry(key).to_string().clone() += 1;
                            // 用正确的写法：
                            // *err_types.entry(key.to_string()).or_default() += 1;
                            println!("FAIL {stem}/{name}: {key} ({} diffs)", diff.lines.len());
                            // 显示前 3 个差异行
                            for dl in diff.lines.iter().take(3) {
                                match dl {
                                    common::DiffLine::Changed { line, expected, actual } =>
                                        println!("  L{line}: exp='{expected}' act='{actual}'"),
                                    common::DiffLine::ExtraExpected { line, content } =>
                                        println!("  L{line}: exp='{content}' act=(missing)"),
                                    common::DiffLine::ExtraActual { line, content } =>
                                        println!("  L{line}: exp=(missing) act='{content}'"),
                                }
                            }
                        }
                    }
```

注意：上面有一个 bug（`*err_types.entry(key).to_string().clone()` 应为 `*err_types.entry(key.to_string()).or_default()`），修正为：

```rust
                    Ok(actual) => {
                        if actual.trim() != expected.trim() {
                            shown += 1;
                            let diff = diff_css(expected.trim(), actual.trim());
                            let key = diff.classify();
                            *err_types.entry(key.to_string()).or_default() += 1;
                            println!("FAIL {stem}/{name}: {key} ({} diffs)", diff.lines.len());
                            for dl in diff.lines.iter().take(3) {
                                match dl {
                                    common::DiffLine::Changed { line, expected, actual } =>
                                        println!("  L{line}: exp='{expected}' act='{actual}'"),
                                    common::DiffLine::ExtraExpected { line, content } =>
                                        println!("  L{line}: exp='{content}' act=(missing)"),
                                    common::DiffLine::ExtraActual { line, content } =>
                                        println!("  L{line}: exp=(missing) act='{content}'"),
                                }
                            }
                        }
                    }
```

- [ ] **步骤 3：编译检查**

运行：`cargo test --test cf_diag --no-run`
预期：编译成功

- [ ] **步骤 4：运行验证**

运行：`cargo test --test cf_diag diag_color -- --nocapture 2>&1 | head -20`
预期：输出中 FAIL 行显示 `content_diff` / `missing_output` 等分类 + 前 3 个差异行

运行：`RUST_LOG="cssdiff=debug" cargo test --test cf_diag diag_color -- --nocapture 2>&1 | head -30`
预期：输出中包含 `cssdiff` target 的 debug 行级差异 events

- [ ] **步骤 5：Commit**

```bash
git add tests/cf_diag.rs
git commit -m "feat: cf_diag 集成 CSS diff — 逐行差异显示替代只看第一行"
```

---

## 任务 6：sass-spec 最小化工具

**文件：**
- 创建：`tests/minimize.rs`

- [ ] **步骤 1：编写最小化工具**

创建 `tests/minimize.rs`：

```rust
//! sass-spec 最小化工具——delta debugging 找到最小复现用例。
//!
//! 用法：
//! ```bash
//! # 看最小化摘要
//! RUST_LOG="minimize=info" cargo test --test minimize minimize_color_error -- --nocapture
//!
//! # 看每次移除尝试
//! RUST_LOG="minimize=debug" cargo test --test minimize minimize_color_error -- --nocapture
//! ```

mod common;

use sasspile::lex::Lexer;
use sasspile::lex::token::Token;
use sasspile::parse::ast::Node;
use sasspile::parse::Parser;
use std::path::Path;

/// 失败判定——决定最小化后是否"仍然失败"。
enum FailOracle<'a> {
    /// 错误模式：编译仍然报错。
    Error,
    /// 输出保持模式：输出与原始（错误）输出相同。
    OutputPreserve { original_output: &'a str },
}

impl<'a> FailOracle<'a> {
    fn still_fails(&self, input: &str) -> bool {
        match self {
            FailOracle::Error => {
                match sasspile::compile_expanded(input) {
                    Ok(css) => {
                        tracing::debug!(
                            target: "minimize",
                            input_len = input.len(),
                            output_len = css.len(),
                            "compiled OK, revert removal"
                        );
                        false
                    }
                    Err(e) => {
                        tracing::info!(
                            target: "minimize",
                            error = %e,
                            input_len = input.len(),
                            "still errors, keep removal"
                        );
                        true
                    }
                }
            }
            FailOracle::OutputPreserve { original_output } => {
                match sasspile::compile_expanded(input) {
                    Ok(css) => {
                        let same = css.trim() == original_output.trim();
                        tracing::info!(
                            target: "minimize",
                            output_unchanged = same,
                            input_len = input.len(),
                            "output comparison"
                        );
                        same
                    }
                    Err(_) => {
                        tracing::warn!(
                            target: "minimize",
                            "compilation failed during output-preserve"
                        );
                        false
                    }
                }
            }
        }
    }
}

/// 解析 SCSS 为 AST 节点列表。
fn parse_to_nodes(input: &str) -> Vec<Node> {
    let tokens: Vec<Token> = Lexer::new(input)
        .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace) | Ok(Token::Eof)))
        .collect::<sasspile::error::Result<Vec<_>>>()
        .unwrap_or_default();
    Parser::parse(&tokens)
        .map(|ast| ast.nodes)
        .unwrap_or_default()
}

/// 最小化 SCSS 输入——delta debugging。
fn minimize(input: &str, oracle: &FailOracle) -> String {
    let mut nodes = parse_to_nodes(input);
    let original_n = nodes.len();

    tracing::info!(
        target: "minimize",
        original_nodes = original_n,
        input_len = input.len(),
        "minimization started"
    );

    let mut changed = true;
    let mut round = 0;

    while changed && nodes.len() > 1 {
        changed = false;
        round += 1;

        tracing::info!(
            target: "minimize",
            round = round,
            n_nodes = nodes.len(),
            "new round"
        );

        let mut i = 0;
        while i < nodes.len() {
            let removed = nodes.remove(i);
            let remaining: String = nodes.iter()
                .map(|n| n.to_scss(0))
                .collect::<Vec<_>>()
                .join("\n");

            tracing::debug!(
                target: "minimize",
                round = round,
                removed_node = ?std::mem::discriminant(&removed),
                remaining_nodes = nodes.len(),
                "trying removal"
            );

            if oracle.still_fails(&remaining) {
                changed = true;
                tracing::info!(
                    target: "minimize",
                    round = round,
                    index = i,
                    remaining_nodes = nodes.len(),
                    "removed node, still fails"
                );
            } else {
                nodes.insert(i, removed);
                tracing::debug!(
                    target: "minimize",
                    round = round,
                    index = i,
                    "reverted removal"
                );
            }
            i += 1;
        }
    }

    let result: String = nodes.iter()
        .map(|n| n.to_scss(0))
        .collect::<Vec<_>>()
        .join("\n");

    tracing::info!(
        target: "minimize",
        original_nodes = original_n,
        final_nodes = nodes.len(),
        original_len = input.len(),
        final_len = result.len(),
        rounds = round,
        "minimization complete"
    );

    result
}

/// HRX 解析——提取所有 (name, input, expected) 三元组。
fn parse_hrx(content: &str) -> Vec<(String, String, String)> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut path = String::new();
    let mut content_buf = String::new();
    for line in content.lines() {
        if line.starts_with("<===>") {
            if !path.is_empty() { files.push((path.clone(), content_buf)); }
            path = line.trim_start_matches("<===>").trim().to_string();
            content_buf = String::new();
        } else {
            content_buf.push_str(line);
            content_buf.push('\n');
        }
    }
    if !path.is_empty() { files.push((path, content_buf)); }
    let mut cases = Vec::new();
    for (p, input) in &files {
        if p.ends_with("input.scss") {
            let base = p.strip_suffix("input.scss").unwrap_or(p).to_string();
            let out_path = format!("{base}output.css");
            let err_path = format!("{base}error");
            let output = files.iter().find(|(pp,_)| pp==&out_path).map(|(_,c)|c.clone()).unwrap_or_default();
            let has_error = files.iter().any(|(pp,_)| pp==&err_path);
            if !has_error && !output.is_empty() {
                cases.push((base.trim_end_matches('/').to_string(), input.clone(), output));
            }
        }
    }
    cases
}

fn collect_hrx(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { collect_hrx(&path, files); }
            else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() < 50_000 { files.push(path); }
                }
            }
        }
    }
}

#[test]
fn minimize_color_error() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
    let dir = spec_root.join("core_functions/color");
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for (name, input, _expected) in &parse_hrx(&content) {
                if sasspile::compile_expanded(input).is_err() {
                    let minimized = minimize(input, &FailOracle::Error);
                    println!("=== {stem}/{name} ===");
                    println!("原始 ({} bytes):\n{input}", input.len());
                    println!("最小化 ({} bytes):\n{minimized}", minimized.len());
                    return; // 只处理第一个错误用例
                }
            }
        }
    }
    println!("no error cases found in core_functions/color");
}

#[test]
fn minimize_extend_error() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
    let dir = spec_root.join("directives/extend");
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for (name, input, _expected) in &parse_hrx(&content) {
                if sasspile::compile_expanded(input).is_err() {
                    let minimized = minimize(input, &FailOracle::Error);
                    println!("=== {stem}/{name} ===");
                    println!("原始 ({} bytes):\n{input}", input.len());
                    println!("最小化 ({} bytes):\n{minimized}", minimized.len());
                    return;
                }
            }
        }
    }
    println!("no error cases found in directives/extend");
}
```

- [ ] **步骤 2：编译检查**

运行：`cargo test --test minimize --no-run`
预期：编译成功

- [ ] **步骤 3：运行最小化测试**

运行：`RUST_LOG="minimize=info" cargo test --test minimize minimize_color_error -- --nocapture 2>&1 | head -30`
预期：输出最小化摘要（original_nodes, final_nodes, rounds）

- [ ] **步骤 4：运行 lib 测试确认无回归**

运行：`cargo test --lib 2>&1 | tail -3`
预期：全部 PASS

- [ ] **步骤 5：Commit**

```bash
git add tests/minimize.rs
git commit -m "feat: sass-spec 最小化工具 — delta debugging + tracing events"
```

---

## 任务 7：颜色转换 + builtin 函数 Events

**文件：**
- 修改：`src/eval/mod.rs`（`hsl_to_rgb`, `hwb_to_rgb`, `rgb_to_hsl`, `builtin_darken`, `builtin_lighten`, `builtin_mix`, `builtin_rgba` 函数）

- [ ] **步骤 1：在颜色转换函数中添加 trace events**

在 `hsl_to_rgb` 函数开头和结尾添加 events：

```rust
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Color {
    let h = h.rem_euclid(360.0);
    tracing::trace!(
        target: "sasspile::color",
        fn = "hsl_to_rgb",
        h = h, s = s, l = l,
        "converting HSL to RGB"
    );
    // ... 现有逻辑不变 ...
    let result = Color::rgb(
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    );
    tracing::trace!(
        target: "sasspile::color",
        fn = "hsl_to_rgb",
        r = result.r, g = result.g, b = result.b,
        "HSL to RGB result"
    );
    result
}
```

在 `rgb_to_hsl` 函数开头和结尾添加类似 events。

在 `hwb_to_rgb` 函数开头添加类似 events。

- [ ] **步骤 2：在颜色 builtin 函数中添加 debug events**

在 `builtin_darken` 中添加：

```rust
fn builtin_darken(args: &[Value]) -> Result<Value> {
    match args {
        [Value::Color(c), Value::Number(amount, _)] => {
            tracing::debug!(
                target: "sasspile::color",
                fn = "darken",
                input_r = c.r, input_g = c.g, input_b = c.b, input_a = c.a,
                amount = *amount,
                "darken input"
            );
            let factor = 1.0 - (*amount as f32 / 100.0);
            let result = Value::Color(Color::rgba(
                (c.r as f32 * factor) as u8,
                (c.g as f32 * factor) as u8,
                (c.b as f32 * factor) as u8,
                c.a,
            ));
            tracing::debug!(
                target: "sasspile::color",
                fn = "darken",
                result = %result,
                "darken result"
            );
            Ok(result)
        }
        _ => Err(SassError::Eval("darken 需要 (color, amount) 参数".into())),
    }
}
```

对 `builtin_lighten`、`builtin_mix`、`builtin_rgba` 添加类似 events。

- [ ] **步骤 3：编译检查**

运行：`cargo check`
预期：无错误

- [ ] **步骤 4：运行验证**

运行：`RUST_LOG="sasspile::color=trace" cargo test --lib test_compile_hsl -- --nocapture`
预期：输出中包含 `sasspile::color` target 的 trace events（HSL → RGB 转换值）

运行：`cargo test --lib 2>&1 | tail -3`
预期：全部 PASS

- [ ] **步骤 5：Commit**

```bash
git add src/eval/mod.rs
git commit -m "feat: 颜色转换/builtin 函数值快照 events — target=sasspile::color"
```

---

## 任务 8：@extend + BinOp Events

**文件：**
- 修改：`src/eval/mod.rs`（`apply_extends` 函数 + BinOp 求值路径）

- [ ] **步骤 1：在 apply_extends 中添加 span + events**

在 `apply_extends` 函数开头添加 span，在关键路径添加 events：

```rust
fn apply_extends(nodes: &mut [CssNode], extends: &[(String, String)]) {
    let span = tracing::info_span!("apply_extends", n_extends = extends.len());
    let _enter = span.enter();

    for node in nodes.iter_mut() {
        match node {
            CssNode::Rule { selector, children, .. } => {
                tracing::debug!(
                    target: "sasspile::extend",
                    selector = %selector,
                    "processing rule for extends"
                );

                for (extender, target) in extends {
                    let target_trimmed = target.trim();
                    if selector.contains(target_trimmed) {
                        tracing::info!(
                            target: "sasspile::extend",
                            extender = %extender,
                            target = %target_trimmed,
                            selector = %selector,
                            "extend matched"
                        );
                        // ... 现有替换逻辑不变 ...

                        if target_trimmed.starts_with('%') {
                            *selector = selector.replace(target_trimmed, extender);
                            tracing::debug!(
                                target: "sasspile::extend",
                                new_selector = %selector,
                                "placeholder replaced"
                            );
                        } else {
                            let new_sel = selector.replace(target_trimmed, extender);
                            if !new_sel.is_empty() && new_sel != *selector {
                                if !selector.contains(&new_sel) {
                                    selector.push_str(", ");
                                    selector.push_str(&new_sel);
                                    tracing::debug!(
                                        target: "sasspile::extend",
                                        final_selector = %selector,
                                        "extender appended"
                                    );
                                }
                            }
                        }
                    }
                }
                Self::apply_extends(children, extends);
                let parts: Vec<&str> = selector.split(',')
                    .filter(|s| !s.trim().starts_with('%'))
                    .collect();
                *selector = parts.join(",").trim().to_string();
            }
            CssNode::AtRule { has_body: true, children, .. } => {
                Self::apply_extends(children, extends);
            }
            CssNode::AtRoot(kids) => {
                Self::apply_extends(kids, extends);
            }
            _ => {}
        }
    }
}
```

- [ ] **步骤 2：在 BinOp 求值中添加 trace events**

找到 `eval_value` 中处理 `Value::BinOp` 的分支，在操作数求值后和结果计算后添加 events：

```rust
// 在 BinOp 求值逻辑中
let l = Self::eval_value(left, env)?;
let r = Self::eval_value(right, env)?;

tracing::trace!(
    target: "sasspile::binop",
    op = ?op,
    left = %l, right = %r,
    "binop operands evaluated"
);

// ... 现有计算逻辑 ...
let result = match op { ... };

tracing::trace!(
    target: "sasspile::binop",
    op = ?op,
    result = %result,
    "binop result"
);
```

注意：具体插入位置取决于 `eval_value` 中 BinOp 的处理方式。如果 BinOp 是在 `eval_value` 内联处理的，在 match 分支内添加 events。如果 BinOp 有单独的辅助函数，在该函数内添加。

- [ ] **步骤 3：编译检查**

运行：`cargo check`
预期：无错误

- [ ] **步骤 4：运行验证**

运行：`RUST_LOG="sasspile::extend=debug" cargo test --lib test_compile_extend -- --nocapture`
预期：输出中包含 `sasspile::extend` target 的 events（匹配成功、选择器替换）

运行：`cargo test --lib 2>&1 | tail -3`
预期：全部 PASS

- [ ] **步骤 5：Commit**

```bash
git add src/eval/mod.rs
git commit -m "feat: @extend 选择器匹配 + BinOp 值快照 events — target=sasspile::extend/sasspile::binop"
```

---

## 自检

### 1. 规格覆盖度

| 设计文档章节 | 实现任务 | 覆盖 |
|-------------|---------|------|
| §2 CSS Diff 模块 | 任务 4 + 任务 5 | ✅ |
| §3 最小化工具 | 任务 2 + 任务 3 + 任务 6 | ✅ |
| §4 color/selector events | 任务 7 + 任务 8 | ✅ |
| §5 init_tracing 升级 | 任务 1 | ✅ |
| §7 文件清单 | 全部 7 个文件 | ✅ |
| §8 不在范围内 | 未涉及 | ✅ |

### 2. 占位符扫描

- 任务 2 中的 `_ => format!(...)` 是临时占位，在任务 3 中被替换为完整实现 ✅
- 无其他 TODO/待定/后续 ✅

### 3. 类型一致性

- `DiffLine` / `DiffResult` 在任务 4 定义，任务 5 引用——名称一致 ✅
- `FailOracle` 在任务 6 定义和使用——名称一致 ✅
- `Node::to_scss(&self, indent: usize)` 签名在任务 2、3、6 中一致 ✅
- Event target 名称在全部任务中一致：`sasspile::color`、`sasspile::extend`、`sasspile::binop`、`cssdiff`、`minimize` ✅

### 4. 模块引用

- `tests/common/mod.rs` 被 `cf_diag.rs`（任务 5）和 `minimize.rs`（任务 6）引用为 `mod common;` ✅
- `tests/common_test.rs` 是临时测试入口（任务 4），任务 5 和 6 不依赖它 ✅
