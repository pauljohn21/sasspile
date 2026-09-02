## Context

sasspile 测试模块依赖 `hrx-auditor` crate（`../scss-rust`）解析 HRX 文件。9 个测试文件各自 `use hrx_auditor::parser::{parse_hrx as hrx_parse, HrxArchive, HrxEntry}; use hrx_auditor::vfs::Vfs;`。

## Goals / Non-Goals

**Goals:**
- 移除 `hrx-auditor` dev-dependency
- 创建内联 `tests/hrx_support.rs` 共享模块
- sass_spec_full.rs 改为非隔离模式（路径前缀 + 共享 VFS）
- 零外部路径依赖

**Non-Goals:**
- 不改 HRX 解析逻辑本身
- 不改 src/ 生产代码
- 不改公开 API

## Decisions

### D1: 内联 hrx_support.rs 模块

将 hrx-auditor 的 `parser.rs`（~120 行）和 `vfs.rs`（~100 行）直接搬到 `tests/hrx_support.rs`，移除 `anyhow` 依赖，`parse_hrx` 返回 `Result<HrxArchive, String>` 兼容旧调用模式。

### D2: 非隔离模式（路径前缀）

旧模式按 `===` 分隔符将 entries 分成独立组，每组构建自己的 VFS。这导致 `@use 'callable/arguments/mixin/utils'` 等跨组引用无法解析。

新模式：所有条目共享同一个 VFS，文件路径加 HRX 文件所在目录作为前缀（如 `callable/arguments/mixin/utils`），使绝对路径 `@use` 能正确解析。

### D3: parse_hrx_to_cases 高级 API

```rust
pub fn parse_hrx_to_cases(content: &str, hrx_rel_path: &str) -> Vec<HrxCase>
```

自动处理路径前缀、文件过滤（.scss/.css/.sass）、input/output/error 匹配。

### D4: 9 个测试文件统一引用

| 文件 | 用途 |
|------|------|
| sass_spec_full.rs | 全量统计（parse_hrx_to_cases + run_case） |
| cf_diag.rs | core_functions 诊断 |
| css_diag.rs | CSS 差异诊断 |
| expr_diag.rs | expressions 诊断 |
| sass_spec.rs | 限制数量测试 |
| diag_detail.rs | 精准诊断 |
| minimize.rs | 最小化工具 |
| cf_color.rs | 颜色诊断（#[ignore]） |
| diag_directives.rs | 指令诊断 |

## Risks

- **非隔离模式下文件路径冲突** → 通过 HRX 目录前缀避免
- **parse_hrx 返回类型变更** → 保持 Result 兼容旧模式
- **跨组引用行为变化** → 更多文件被正确解析，总用例数增加

## 改动范围

| 文件 | 改动 |
|------|------|
| `tests/hrx_support.rs` | 新建（449 行）|
| `tests/sass_spec_full.rs` | 重写（引用 hrx_support） |
| `tests/cf_diag.rs` | 替换 use 行 |
| `tests/css_diag.rs` | 替换 use 行 |
| `tests/expr_diag.rs` | 替换 use 行 |
| `tests/sass_spec.rs` | 替换 use 行 |
| `tests/diag_detail.rs` | 替换 use 行 |
| `tests/minimize.rs` | 替换 use 行 |
| `tests/cf_color.rs` | 替换 use 行 |
| `tests/diag_directives.rs` | 替换 use 行 |
| `Cargo.toml` | 删除 hrx-auditor dev-dependency |
| `AGENTS.md` | 更新 HRX 解析架构章节 |
| `docs/CODE_INDEX.md` | 更新 HRX 解析条目 |
