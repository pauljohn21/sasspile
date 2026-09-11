## Context

sasspile 近几天经过多轮 AI 迭代新增了 color_change.rs、color_hwb.rs、color_scale.rs、selector_append.rs、selector_nest.rs、selector_ops.rs、display_color_spaces.rs 等文件，并对 builtin.rs、color.rs、color_adjust.rs、color_hwb_hsl.rs、display_color.rs、extend.rs、mod.rs 进行了修改。这些变更中混入了违反 AGENTS.md §7 函数式强制规则的过程式代码。

当前约束：
- 纯 Rust 2024 edition，toolchain 1.97
- 测试基线：核心 202/202 + sass-spec 统计不退化
- 重构必须保持输入输出语义完全不变

## Goals / Non-Goals

**Goals:**
- 消除所有 `for + push/extend` 可变累积，替换为迭代器链 (`map`/`fold`/`flat_map`/`collect`)
- 消除 `match` 内嵌 `return` 混用，替换为 `?` 或 `if let` 链
- 消除重复 `let a/b/c = same_pattern(x)` 模式，替换为 `zip` + `map` 通道归一化
- 展平嵌套 `match` 为单一分派
- 提取重复逻辑为辅助函数

**Non-Goals:**
- 不改变任何编译器输入输出语义
- 不引入新依赖
- 不重构与本次变更无关的历史代码
- 不改变模块组织结构

## Decisions

### D1: parse_calc_args 状态机 → 分割函数

**决策**：保持 for 循环但提取为 `split_top_level` 纯函数，用 `&str` 输入返回 `Vec<&str>`。

**理由**：括号深度跟踪本质是有状态解析，强行改为迭代器链会丧失可读性。折中方案：for 循环封入纯函数（输入 `&str`，输出 `Vec<&str>`），上层用迭代器 `map(parse_calc_arg_value)` 收集。

**替代方案**：❌ 强行用 `fold` + `String` 累积 → 更复杂，无可读性收益。

### D2: call_hwb 链式 match → `?` 管道 + 枚举通道

**决策**：使用 `Result<Option<T>>` 链式 `?` 传播替代连续 `match + return`。

**理由**：`Result<Option<_>>` 天然适合 "解析失败透传 → 数值缺失透传 → 构造颜色" 三步管道。

### D3: split_simple_selectors → 标准库 `split` 技巧

**决策**：用 `fold` + ` split_off` 风格或 `peekable` 迭代器替代 for + push。

**理由**：需要追踪当前累积段，`peekable` + `fold` 是自然的函数式方案。

### D4: call_extend/call_replace 重复 match → 提取 `validate_args`

**决策**：提取 `fn validate_extend_args(args: &[Value]) -> Result<()>` 统一校验。

**理由**：两函数的参数校验逻辑完全相同（match args.len() < 3 + > 3），DRY。

### D5: builtin_rgba 百分比转换 → 提取 `normalize_rgb_channel`

**决策**：提取 `fn normalize_channel(val: f64, unit: Option<&str>) -> f64` 统一百分比转换。

**理由**：3 次重复 if-else 转换（r/g/b 各一次）可用一次 `map` 迭代器替代。

### D6: adjust_legacy 重复 let → 迭代器 zip

**决策**：使用 `keys.iter().zip(channels.iter()).map(...).collect::<Vec<_>>()` 替代 3+ 独立 let。

**理由**：结构完全相同的操作适合用 zip + map 模式。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| parse_calc_args 重构引入边界错误 | `cargo test --test interp_test` 包含 calc 解析用例 |
| call_hwb Result 类型不匹配 | 仔细检查 `?` 与 `Ok(Some(...))` 嵌套层级 |
| split_simple_selectors 字符边界错误 | selector-spec 覆盖 `selector-simple-selectors` |
| 重构导致性能下降 | 全部为 unpacking 零新增分配 |
| sass-spec 统计不退化 | 重构后跑 `SPEC_STORE_CMD=run` 对比 |
