## Why

sasspile 当前 `Cargo.toml` 缺少 `[lints]` 段，`cargo clippy --workspace` 因 `never_loop` error 编译失败，且有 9 个默认级别 warning。启用 `clippy::all` + `clippy::pedantic` 后暴露 928 个 pedantic 警告，分布在颜色转换矩阵常量（`unreadable_literal` 203 个）、单字符变量名（`single_char_names` 113 个）、cast 安全性（92 个）、文档缺失（90+24 个）等方面。项目需要建立编译器级别的质量基线，在 CI 中强制执行，防止代码退化。

## What Changes

- **修复编译阻断**：`src/eval/value/calc.rs:188` 的 `never_loop` error——`strip_parens` 的 `while` 循环末尾无条件 `break` 导致循环只执行一次
- **添加 Cargo.toml `[lints]` 段**：启用 `clippy::all = "warn"` + `clippy::pedantic = "warn"` + `unsafe_code = "warn"`
- **批量修复机械性 pedantic 警告**（约 400 个）：`unreadable_literal`（加 `_` 分隔符）、`redundant_closure`、`match_like_matches_macro`、`needless_lifetimes`、`manual_strip`、`unnecessary_map_or`、`needless_question_mark`、`unnested_or_patterns`、`format_push_string`、`directly_string_format`
- **修复 cast 安全性**（92 个）：`as u8` → `u8::try_from`/`clamp`，`as f64` → `f64::from`，`as usize` → `usize::try_from`
- **合并相同 match 臂**（58 个 `match_same_arms`）
- **消除 wildcard import**（37 个）：改为显式 import
- **完善文档**（90 个 `doc_markdown` + 24 个 `missing_errors_doc`/`missing_panics_doc`）：技术术语加反引号，`Result` 返回函数补 `# Errors` 段
- **选择性 allow 颜色模块噪声**：`color_conv.rs` 等颜色转换模块中 `single_char_names`（`r/g/b/h/s/l` 是标准命名）和颜色常量 `unreadable_literal` 通过模块级 `#![allow(...)]` 处理
- **清理默认 clippy 警告**（9 个）：`unused_import`、`type_complexity`、`redundant_closure`、`manual_strip`、`unnecessary_map_or`、`match_like_matches_macro`、`needless_question_mark`、`needless_lifetimes`

## Capabilities

### New Capabilities

- `lint-policy`: 编译器 lint 配置策略——定义 `Cargo.toml` `[lints]` 段标准配置、模块级 allow 规则、CI 强制检查流程

### Modified Capabilities

（无——本次变更不修改任何现有 spec 的行为要求）

## Impact

- **Cargo.toml**：新增 `[lints.rust]` 和 `[lints.clippy]` 段
- **src/eval/value/calc.rs**：修复 `strip_parens` never_loop bug
- **src/lib.rs**：移除 unused import
- **src/eval/builtin/color_conv.rs**：添加模块级 allow，修复 cast
- **src/eval/builtin/color.rs**、**color_adjust.rs**、**color_gamut.rs** 等：修复 cast、合并 match 臂
- **src/eval/builtin/math_trig.rs**、**math.rs**：修复 redundant closure、 needless lifetimes
- **src/eval/rule.rs**：修复 manual_strip
- **src/css/mod.rs**：修复 type_complexity
- **src/eval/value/mod.rs**：修复 needless_question_mark
- **全 crate 约 40 个文件**：doc_markdown 反引号、wildcard import 显式化
- **CI**：`cargo clippy --workspace` 成为必须通过的检查
