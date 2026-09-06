## Why

经过多次 AI 迭代开发后，sasspile 代码库积累了大量违反 AGENTS.md 规则的代码异味：11 处 unwrap() 会导致运行时 panic、7 个文件超过 500 行上限（最长 636 行）、362 处 clone() 滥用、约 15 处 for+push 命令式累积。更重要的是，现有的 AGENTS.md 静态规则无法阻止 AI 在长上下文中退化——需要建立 CI 层面的自动化检测机制，让违规在合并前就被拦截。

## What Changes

- **消除 11 处 unwrap()**：将 css/selector_parser.rs（7 个）、eval/value/calc_ast.rs（3 个）、eval/value/calc_simplify.rs（1 个）中的 unwrap() 替换为带错误传播的 `?` / `expect()` / `match`
- **拆分 4 个超限文件**：
  - `parse/ast/display.rs` 636 行 → 拆出颜色序列化 → `parse/ast/color_display.rs`
  - `eval/builtin/color.rs` 627 行 → 拆分通道读取 / HSL-HWB / adjust-change-scale / 灰度反相
  - `eval/builtin/color_adjust.rs` 614 行 → 拆分现代空间 vs 传统空间
  - `css/mod.rs` 549 行 → 拆出 `merge_at_rules` → `css/merge.rs`
- **Clippy deny 级 lint**：`unwrap_used = "deny"` + `todo = "deny"` + `unimplemented = "deny"`
- **文件行数检测脚本**：`tests/file_size_check.rs` — 测试失败时列出超限文件

## Capabilities

### New Capabilities
- `unwrap-safety`: 解析器路径不允许 unwrap()，必须返回 Result 或使用 expect() 提供上下文
- `file-size-guard`: 所有 src/**/*.rs 文件 ≤ 500 行，超限必须在 CI 中报告
- `clippy-deny-lints`: Cargo.toml / clippy.toml 配置 deny 级 lint 列表

### Modified Capabilities
（无 spec 级行为变更，仅清理和拆分）

## Impact

- **影响文件**：
  - 修改：`css/selector_parser.rs`、`eval/value/calc_ast.rs`、`eval/value/calc_simplify.rs`
  - 修改并重命名/拆分：`parse/ast/display.rs`、`eval/builtin/color.rs`、`eval/builtin/color_adjust.rs`、`css/mod.rs`
  - 新增：`parse/ast/color_display.rs`、`eval/builtin/color_*.rs`（多个子模块）、`css/merge.rs`
  - 修改：`Cargo.toml`（clippy 配置）
- **影响 API**：公开 API 不变（仅内部重构）
- **sass-spec 影响**：预期无回归（仅 eliminat panic + 文件拆分）
