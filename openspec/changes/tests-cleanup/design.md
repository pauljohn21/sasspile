## Context

tests/ 目录积累了 54 个测试文件，12 个诊断工具重复定义 `parse_hrx()` / `compile_case()` / `collect_hrx()` 相同逻辑（各 ~80 行 × N）。调试会话产生的临时文件残留。compile_test.rs 因持续添加新功能增至 551 行超限。

核心约束：每个 `.rs` 文件是独立 crate（`mod hrx_support;` 重复内联），所以无法直接跨文件共享函数——需要在合并后的单一文件中集中实现共享逻辑一次。

## Goals / Non-Goals

**Goals:**
- 12 个诊断文件 → 1 个 `diagnostic_runner.rs`，共享 parse/compile 逻辑只写一次
- 删除所有临时/被替代文件
- 所有文件 ≤ 500 行
- 不丢失任何测试覆盖

**Non-Goals:**
- 不改变 src/ 任何代码
- 不修改测试的测试逻辑/断言
- 不引入新的测试框架或依赖

## Decisions

### Decision 1: 诊断工具合并为单一文件

**选择**: 将 cf_diag、cfs_diag、cfs_units、css_diag、diag_detail、diag_directives、expr_diag、use_diag 合并为 `diagnostic_runner.rs`

**当前 12 个文件共享的重复模式:**
```rust
// 每个文件都重新定义：
struct HrxCase { files, input_path, expected_output, expect_error }
fn parse_hrx(content: &str) -> Vec<HrxCase> { ... } // ~40 行
fn collect_hrx(dir, files) { ... }                  // ~10 行
fn compile_case(case, spec_root, hrx_stem)          // ~25 行
```

**合并方案**: 
- 一份共享的 `HrxCase` 结构体 + `parse_case()` + `compile_case()` + `collect_hrx_files()`
- 按模块组织测试函数：`mod core_functions`; `mod directives`; `mod values`; `mod css_mod`
- 每个原有的 `#[test]` 函数按主题保留

**备选**: 提取为共享库 → 不可行（独立 crate 限制）
**备选**: 保持独立但提取 `mod shared_diag;` → 增加一个额外文件，合并更彻底

### Decision 2: compile_test.rs 拆分边界

**选择**: 抽出 CSS Color Level 4 的 14 个测试到 `compile_color_test.rs`

**理由**: compile_test.rs 的 551 行中，~167 行是色彩空间序列化验证，属于独立关注点。抽出后 compile_test.rs ~380 行聚焦编译管线核心功能。

**不选**: 按"功能"进一步拆分 → 过度工程，2 文件是最佳粒度

### Decision 3: hwb_spec_diag.rs 整合方式

**选择**: 将 3 个测试函数移入 `compile_test.rs`，删除原文件

**理由**: compile_test.rs 已有测试 compile_color_* 的模式，hwb_degenerate_hue 等与颜色输出验证同源

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| 诊断文件合并后单个文件过大 | 严格控制 ≤500 行；预计 ~400 行 |
| 原 `cargo test --test cf_diag --name diag_list` 路径改变 | 新路径 `cargo test --test diagnostic_runner -- diag_list` |
| 遗漏某个测试函数的移植 | 建立 checklist 对照表，逐个迁移验证 |

## Migration Plan

**Phase 1: 删除临时文件** (无依赖)
- 删除 extend_debug.rs、trace_table.rs、failures_inspect.rs、failures_analyze.rs

**Phase 2: 合并诊断工具** (核心)
- 新建 diagnostic_runner.rs，从 cf_diag.rs 和 cfs_diag.rs 提取共享逻辑
- 逐个迁移其余诊断文件的测试函数

**Phase 3: 拆分 compile_test.rs**
- 抽出 14 个色彩空间测试 → compile_color_test.rs
- 合并 hwb_spec_diag.rs 的 3 个测试 → compile_test.rs
- 删除 hwb_spec_diag.rs
- 合并 file_size_check.rs → compile_test.rs

**Phase 4: 验证**
- `cargo test --test compile_test` 全通过
- `cargo test --test diagnostic_runner` 全通过
- `cargo test --test sass_spec_full` 统计不退化
- `cargo test --test failures_json` 正常

## Open Questions

- `use_diag.rs` 的 2 个测试函数（test_use_subdirs, test_use_top_level_hrx）是否融入 diagnostic_runner？→ **是，作为 `mod directives::use_diag`**
- `diag_directives.rs` 的 forward/extend 诊断是否保留？→ **是，作为 `mod directives::forward_extend`**
- `cfs_units.rs` 的 hsl_detail 测试如何处理？→ **合并为 diagnostic_runner 的一个统计测试**
