## Why

sasspile 通过 `dev-dependency` 引用 `hrx-auditor` crate（路径 `../scss-rust`），用于解析 sass-spec 的 HRX 测试文件。这造成了两个问题：

1. **外部路径依赖**：`Cargo.toml` 中 `hrx-auditor = { path = "../scss-rust" }` 依赖项目外目录，不利于独立构建
2. **代码分散**：HRX 解析逻辑在 `scss-rust` 项目中，sasspile 测试无法自包含

HRX 解析逻辑本身很简单（~120 行 parser + ~100 行 VFS），完全可以内联到 sasspile 测试模块中。

## What Changes

1. **创建 `tests/hrx_support.rs`**：内联 `parse_hrx`、`HrxArchive`、`HrxEntry`、`Vfs`、`VfsNode`，从 hrx-auditor 抄过来，移除 `anyhow` 依赖，改用 `Result<T, String>`
2. **重写 `tests/sass_spec_full.rs`**：废弃 `===` 分组隔离逻辑，改为非隔离模式（所有条目共享同一 VFS，路径加 HRX 名作前缀），使 `@use` 跨组引用能正确解析
3. **修改 9 个测试文件**：将 `use hrx_auditor::...` 替换为 `mod hrx_support; use hrx_support::...`
4. **移除 `Cargo.toml` 中的 `hrx-auditor` dev-dependency**
5. **更新文档**：AGENTS.md、docs/CODE_INDEX.md

## Impact

- sass-spec 基线：3216/5624 = 57%（非隔离模式 + 路径前缀）
- compile_test：43/43 ✅
- ep_full：121/121 ✅
- 零外部路径依赖，sasspile 完全自包含
