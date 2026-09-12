## Why

AI 多次迭代后，新增和修改的 15 个文件中混入了过程式风格代码，违反 sasspile 函数式 Rust 强制规则（AGENTS.md §7）。这些反模式包括 `for + push` 可变累积、`match + return` 混用、重复 let 赋值等，降低代码可读性并增加维护成本。本次清理将全部重构为统一函数式风格。

## What Changes

- **P0（核心违规，4 处）**：
  - `src/eval/builtin.rs:parse_calc_args` — `for` + `push` 可变累积 → 递归/`fold` 或分割函数
  - `src/eval/builtin.rs:parse_calc_arg_value` — `match` + `return` 混用 → `if let` 链 + 表达式
  - `src/eval/builtin/color_hwb.rs:call_hwb` — 4× 连续 `match` + early return → `?` 传播 + 管道
  - `src/eval/builtin/selector_ops.rs:split_simple_selectors` — `for` + `push` 可变累积 → 迭代器链
- **P1（结构性问题，5 处）**：
  - `src/eval/builtin.rs:merge_params_impl` — `&mut result` + `extend_from_slice` → `chain` + `collect`
  - `src/eval/builtin/color_change.rs:change_legacy` — 150 行嵌套 match → 拆分为小组函数
  - `src/eval/builtin/selector_ops.rs:call_extend` + `call_replace` — 重复 match 模式 → 提取参数校验
  - `src/eval/color.rs:builtin_rgba` — 150+ 行百分比转换 → 抽取通道归一化辅助
  - `src/eval/builtin/color_adjust.rs:adjust_modern_rgb_space` — 重复 `let r/g/b` → 迭代器 zip map
- **P2（优化级，6 处）**：
  - `src/eval/builtin.rs:parse_number_with_unit` — `for` 搜索 → `char_indices` + `find`
  - `src/eval/builtin/selector_nest.rs:cartesian_replace` — `fold` 累积 → 乘积迭代器
  - `src/parse/ast/display_color.rs:RgbPercent` 分支 — 嵌套 match → 展平 + 辅助
  - `src/eval/builtin/color_hwb_hsl.rs:merge_named_color_args` — `for` + `push` → 迭代器链
  - `src/parse/ast/display_color_spaces.rs` — 重复 alpha 模式 → `write_channeled` 辅助

## Capabilities

### New Capabilities

（无新增能力——纯内部重构）

### Modified Capabilities

（无 spec 级别行为变更——纯代码风格重构，不影响编译器输入输出语义）

## Impact

- **受影响文件**：10 个（7 个新增 + 3 个修改）
- **改动行数**：约 600 行重构
- **风险**：低——纯风格重构，不改变逻辑；所有测试必须全绿
- **测试要求**：核心测试 202/202 + sass-spec 全量统计不退化
