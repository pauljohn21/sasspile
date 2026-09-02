# Tasks — hrx-auditor-removal

## 1. 创建内联 hrx_support.rs 模块

- [x] 1.1 创建 `tests/hrx_support.rs`
- [x] 1.2 实现 `parse_hrx` — 返回 `Result<HrxArchive, String>`
- [x] 1.3 实现 `HrxArchive`、`HrxEntry` 结构体
- [x] 1.4 实现 `Vfs`、`VfsNode` — `from_archive` + `walk`
- [x] 1.5 实现 `parse_hrx_to_cases` — 高级 API（路径前缀 + 文件过滤）
- [x] 1.6 实现 `run_case` — 写入临时目录 + 编译 + 比对
- [x] 1.7 实现 `parse_hrx_legacy` — 兼容旧 `ParsedHrx` 接口
- [x] 1.8 添加 `#![allow(dead_code)]` 消除跨 crate 误报

## 2. 重写 sass_spec_full.rs

- [x] 2.1 引用 `mod hrx_support` + `mod spec_manifest`
- [x] 2.2 使用 `parse_hrx_to_cases` + `run_case` 替代内联逻辑
- [x] 2.3 非隔离模式（路径加 HRX 名前缀）
- [x] 2.4 临时目录加入 load_paths（使 `@use` 绝对路径能解析）

## 3. 修改 9 个测试文件

- [x] 3.1 `cf_diag.rs` — 替换 `use hrx_auditor::...` 为 `mod hrx_support; use hrx_support::...`
- [x] 3.2 `css_diag.rs` — 同上
- [x] 3.3 `expr_diag.rs` — 同上
- [x] 3.4 `sass_spec.rs` — 同上
- [x] 3.5 `diag_detail.rs` — 同上
- [x] 3.6 `minimize.rs` — 同上
- [x] 3.7 `cf_color.rs` — 同上
- [x] 3.8 `diag_directives.rs` — 同上

## 4. 移除 hrx-auditor 依赖

- [x] 4.1 `Cargo.toml` 删除 `hrx-auditor = { path = "../scss-rust" }`
- [x] 4.2 `cargo check --tests` 编译通过

## 5. 验证 + 文档

- [x] 5.1 `cargo test --test compile_test` — 43/43 ✅
- [x] 5.2 `cargo test --test ep_full` — 121/121 ✅
- [x] 5.3 `cargo test --test sass_spec_full` — 3216/5624 = 57% ✅
- [x] 5.4 更新 AGENTS.md HRX 解析架构章节
- [x] 5.5 更新 docs/CODE_INDEX.md HRX 条目
- [x] 5.6 `codegraph sync`
