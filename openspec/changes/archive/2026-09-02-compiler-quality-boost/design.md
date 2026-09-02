## Context

sasspile 当前有 17,543 行源代码分布在约 50 个 `.rs` 文件中。`Cargo.toml` 没有 `[lints]` 段，`cargo clippy --workspace` 因 `never_loop` error 编译失败。启用 `clippy::pedantic` 后暴露 928 个警告，集中在以下区域：

- 颜色转换矩阵常量（`color_conv.rs` 等）：`unreadable_literal` 203 个、`single_char_names` 113 个
- cast 安全性：`cast_truncation`/`cast_sign_loss` 92 个
- 文档：`doc_markdown` 90 个、`missing_errors_doc`/`missing_panics_doc` 24 个
- 模式匹配：`match_same_arms` 58 个、`redundant_closure` 28 个
- 导入：`wildcard_imports` 37 个

当前测试基线：202/202 核心测试通过，sass-spec 3216/5624 = 57%。

## Goals / Non-Goals

**Goals:**

- `cargo clippy --workspace` 零 error 零 warning（默认级别）
- `cargo clippy --workspace -- -W clippy::pedantic` 零 warning（或通过 `allow` 豁免的合理噪声）
- `Cargo.toml` 有标准 `[lints]` 段，CI 可强制执行
- `never_loop` bug 修复，`strip_parens` 正确处理多层括号
- 不引入任何测试回归（202/202 + sass-spec 基线不退化）

**Non-Goals:**

- 不重构架构或改变函数式管线设计
- 不修改 sass-spec 通过率（仅修 lint，不修功能）
- 不拆分大文件（当前所有文件 ≤ 500 行，已在限内）
- 不添加新功能

## Decisions

### D1: Lint 配置策略——全量 pedantic + 模块级 allow

**选择**：`Cargo.toml` 中 `clippy::all = "warn"` + `clippy::pedantic = "warn"`，对噪声 lint 在模块级 `#![allow(...)]` 而非 crate 级。

**理由**：crate 级 `allow` 会隐藏非颜色模块中的同类问题。模块级 `allow` 只在确实合理的文件上生效。

**替代方案**：crate 级 `allow` 列表——简单但会全局抑制，可能掩盖非颜色代码中的同类问题。已否决。

### D2: 颜色模块 allow 范围

**选择**：在以下文件添加模块级 `#![allow(...)]`：

| 文件 | allow lint | 理由 |
|------|-----------|------|
| `src/eval/builtin/color_conv.rs` | `unreadable_literal`, `single_char_names`, `excessive_precision` | W3C 有理数矩阵系数，标准数学命名 |
| `src/eval/builtin/color_adjust.rs` | `single_char_names` | HSL/HWB 参数 `h/s/l` 是标准命名 |
| `src/eval/builtin/color.rs` | `single_char_names` | RGB `r/g/b` 是标准命名 |
| `src/eval/color_names.rs` | `unreadable_literal` | 颜色常量表 |
| `src/parse/ast/color_types.rs` | `unreadable_literal` | 颜色类型定义 |
| `src/parse/ast/named_colors.rs` | `unreadable_literal` | 命名颜色表 |

**替代方案**：重命名 `r` → `red`、`g` → `green`——会让数学公式不可读，且与 W3C/CSS 规范不一致。已否决。

### D3: cast 修复策略

**选择**：按安全分类处理：

| cast 模式 | 修复方式 | 示例 |
|-----------|---------|------|
| `as u8`（值域 0-255） | `u8::try_from(x).unwrap_or(255)` 或 `.clamp(0, 255) as u8` | 颜色分量 |
| `as f64`（整数→浮点） | `f64::from(x)` | 索引/计数→浮点 |
| `as usize`（i32/usize 转换） | `usize::try_from(x).unwrap_or(0)` | 索引 |
| `as i64` | `i64::from(x)` 或 `try_from` | 大整数 |

**替代方案**：全局 `allow(clippy::cast_*)`——不安全，会隐藏真正的截断 bug。已否决。

### D4: never_loop 修复方式

**选择**：将 `strip_parens` 的 `while` 循环改为 `if`（循环确实只应执行一次——外层括号去除后，内层会通过递归调用处理）。

**理由**：分析 `strip_parens` 逻辑——循环体的 `break` 在 `if !ok || depth != 0` 和 `if Self::parse_simple_number(...).is_some()` 两个分支中，末尾的 `break` 处理的是"括号合法但不是简单数字"的情况，此时应直接返回原始字符串。循环确实不应第二次执行。

**替代方案**：把末尾 `break` 改成 `continue`——会导致无限循环，因为去掉一层括号后如果仍以 `(` 开头，会不断尝试。已否决。

### D5: 批量修复执行顺序

**选择**：按风险从低到高执行：

```
Phase 1: 编译修复（never_loop + Cargo.toml lint 段）
         → cargo clippy --workspace 必须通过
         → 运行全部测试确认无回归

Phase 2: 机械性修复（unreadable_literal, redundant_closure,
         match_like_matches_macro, needless_lifetimes, manual_strip,
         unnecessary_map_or, needless_question_mark, unnested_or_patterns,
         format_push_string, directly_string_format）
         → 每批修复后运行 cargo test

Phase 3: 人工审查修复（cast安全性, match_same_arms,
         wildcard_imports, items_after_statements, unnecessary_wraps）
         → 每个文件修复后运行该文件相关的测试

Phase 4: 文档完善（doc_markdown, missing_errors_doc, missing_panics_doc,
         must_use_candidate, return_self_not_must_use）
         → 文档修改不影响编译，最后处理

Phase 5: 模块级 allow 添加 + 最终验证
         → cargo clippy --workspace -- -W clippy::pedantic 零 warning
         → 全量测试通过
```

## Risks / Trade-offs

- **[Risk] cast 修复改变运行时行为** → 每批 cast 修复后运行颜色相关测试（`compile_test` + `bs_spec`），确认输出不变
- **[Risk] match_same_arms 合并可能改变逻辑** → 只合并 `_ =>` 前的相同臂，不合并涉及副作用的表达式
- **[Risk] 模块级 allow 过多** → 限制 allow 到颜色相关文件，非颜色文件不添加任何 allow
- **[Trade-off] 部分 pedantic lint（`must_use_candidate`）修改面广** → 可在 Phase 4 评估，如果工作量过大可 `allow` 到 crate 级
