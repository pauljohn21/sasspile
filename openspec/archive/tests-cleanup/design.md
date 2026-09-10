## Context

tests/ 目录积累了 54 个测试文件，15 个诊断工具重复定义 `parse_hrx()` / `compile_case()` / `collect_hrx()` 相同逻辑（各 ~80 行 × N）。调试会话产生的临时文件残留。compile_test.rs 因持续添加新功能增至 551 行超限。

核心约束：每个 `.rs` 文件是独立 crate（`mod hrx_support;` 重复内联），所以无法直接跨文件共享函数——需要在合并后的单一文件中集中实现共享逻辑一次。

## Goals / Non-Goals

**Goals:**
- 15 个诊断文件 → 拆分为 `diag_helper.rs`（共享）+ `diagnostic_runner.rs` + `diag_color.rs`，所有文件 ≤ 500 行
- 删除所有临时/被替代文件
- **所有文件 ≤ 500 行**（硬性约束）
- 不丢失任何测试覆盖

**Non-Goals:**
- 不改变 src/ 任何代码
- 不修改测试的测试逻辑/断言
- 不引入新的测试框架或依赖

## Decisions

### Decision 1: 诊断工具拆分为 3 文件（500 行约束）

**选择**: 将 cf_diag、cfs_diag、cfs_diag2、cfs_units、css_diag、diag_detail、diag_directives、diag_hsl2、diag_hsl3、expr_diag、extend_debug、failures_analyze、failures_inspect、trace_table、use_diag 拆分为 `diag_helper.rs`（公共辅助函数）+ `diagnostic_runner.rs`（core_functions + directives + css + expr + use 测试函数）+ `diag_color.rs`（颜色相关诊断 + values + HSL）

500 行约束分析:
- 原 `cf_diag.rs` 本身 462 行
- 原 `use_diag.rs` 189 行
- 12 个文件总和 ~2800 行
- 公共部分提取后，剩余 ~1500 行测试代码
- **不可能全部装入一个 ≤500 行文件**
- 拆分为 3 个文件符合「单一 gather_hrx/parse_case/compile_case 源」的设计意图

**共享方案**:
- `diag_helper.rs` 定义所有公共辅助函数，标记 `pub mod hrx_support / common / spec_manifest` + `pub mod prelude`
- 各测试文件通过 `#[path = "diag_helper.rs"] mod diag_helper;` + `use diag_helper::prelude::*;` 导入

**备选**: 提取为共享库 → 不可行（独立 crate 限制）

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
| 原 `cargo test --test cf_diag --name diag_list` 路径改变 | 新路径 `cargo test --test diagnostic_runner -- diag_list` |
| `#[path = "diag_helper.rs"]` 可能导致 IDE 解析问题 | 编译无误即可，IDE 警告非阻塞 |
| `cargo test --test` 仍需单独执行 | 原 `cf_diag` 等 target 已不存在 |

## Migration Plan

**Phase 1: 删除临时文件** (已完成)
- ~~删除 extend_debug.rs、trace_table.rs、failures_inspect.rs、failures_analyze.rs~~

**Phase 2: 合并诊断工具** (已完成)
- 新建 `diag_helper.rs`（公共 ~200 行）
- 新建 `diagnostic_runner.rs`（core_functions / directives / css / expr / use 测试）
- 新建 `diag_color.rs`（color change/scale/hsl + HSL 统计 + values diag_output_mismatch）
- 删除 11 个旧诊断源文件

**Phase 3: 拆分 compile_test.rs** (已完成)
- 抽出 14 个色彩空间测试 → compile_color_test.rs ✅
- 合并 hwb_spec_diag.rs 的 3 个测试 → compile_test.rs ✅
- 合并 file_size_check.rs → compile_test.rs ✅
- 删除 hwb_spec_diag.rs ✅
- 删除 file_size_check.rs ✅

**Phase 4: 验证**
- `cargo test --test compile_test` 全通过
- `cargo test --test diagnostic_runner` 全通过
- `cargo test --test sass_spec_full` 统计不退化
- `cargo test --test failures_json` 正常
