## Why

AI 多次迭代后，sasspile 项目积累了以下违规项：(1) `tests/specstore/mod.rs` 中 4 个 `println!` 导致 clippy 编译失败；(2) 11 个源文件超出 500 行上限（最严重 `selector_extend.rs` 达 873 行）；(3) lib 级 clippy 21 个 warning（unused imports/variables/functions、`let...else` 可重写等）。本次变更是对这些累积违规的系统性清理，恢复项目到 "零 clippy error + 文件行数合规" 状态。

## What Changes

- **编译阻塞修复**：`tests/specstore/mod.rs` 中 4 处 `println!` → `tracing::info!`
- **clippy fix 自动修复**：variables in format! 内联、unused 项清理、boolean-to-int `match` 化、`let...else` 重写
- **文件拆分**（按优先级）：
  - `src/css/selector_extend.rs` (873→≤500+≤500+剩余) — 拆出 extend/unify/format 子模块
  - `src/eval/reactor.rs` (659→≤500+≤159) — 拆出类型状态定义与方法实现
  - `src/eval/builtin/color_adjust.rs` (636→≤500+≤136) — 拆出 adjust/change/scale
  - `src/eval/builtin/selector.rs` (603→≤500+≤103) — 拆出各选择器函数
  - `src/eval/builtin/color_hwb_hsl.rs` (583→≤500+≤83) — 拆出 HSL/HWB 函数族
  - `src/parse/ast/display_color.rs` (567→≤500+≤67) — 拆出各空间序列化
- **后缀可选项**（如时间充裕）：`module.rs` (540)、`serialize.rs` (539)、`env_impl.rs` (517)、`calc.rs` (505)、`list.rs` (505) 拆分

## Capabilities

### New Capabilities
- `_cli-output-pattern`: spec-store CLI 工具的标准输出模式分离（stats/trend/link/diff 返回 String，由调用者决定如何展示）

### Modified Capabilities
- 无 — 本次为纯清理变更，不改变任何 Sass 语言行为或公共 API

## Impact

- **编译**：clippy 零 error（当前 4 个 blocking error）
- **文件结构**：~6-11 个文件拆分，新增 ~6 个子模块文件
- **测试**：无新增功能测试，依赖现有 243/243 spec 回归
- **sass.spec 通过率**：预期不变（纯重构不改变语义）
- **依赖**：无新增依赖
