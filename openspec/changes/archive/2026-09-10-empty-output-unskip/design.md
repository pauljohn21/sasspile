## Context

sass-spec 测试框架当前将"期望输出为空 + 非错误"的 case 标记为 SKIP。这导致：
- `variables/` 目录 11/14 case 被跳过（通过率 21% 而非 100%）
- 全局 2620 个 case 被跳过（`directives/use` 300、`directives/forward` 300 等）
- 这些 case 实际验证"正确编译但不产生 CSS"的行为，跳过等于失去回归防护

三处 skip 逻辑分布：
1. `tests/specstore/runner.rs:42` — spec_store 运行器
2. `tests/hrx_support.rs:378` — sass_spec_full 运行器
3. `tests/sass_spec_full.rs:91` — 统计计数器

## Goals / Non-Goals

**Goals:**
- 空输出 case 实际编译并验证输出为空字符串
- `variables/` 目录达到 100% 通过率
- 全局新增 ~2600 个有意义的回归测试

**Non-Goals:**
- 不修改编译器行为（只改测试框架）
- 不处理 `.sass` 缩进语法过滤（独立问题）
- 不新增功能或修复编译器 bug

## Decisions

### 决策 1: 移除 skip 而非加白名单

**选择**: 直接删除 `if expected_output.is_empty() && !expect_error { skip }` 分支，不引入目录白名单。

**替代方案**: 加白名单只对特定目录（如 `variables/`）取消 skip — 选择不采用，因为：
- 逻辑应统一：空输出 case 在所有目录都应被验证
- 白名单引入特殊逻辑，增加维护负担
- `directives/use` 等目录的空输出 case 同样有价值

**影响**: 全局 2620 个 case 从 SKIP 变为实际评估。

### 决策 2: 三处同步修改

**选择**: `runner.rs`、`hrx_support.rs`、`sass_spec_full.rs` 三处同步移除 skip 逻辑。

**替代方案**: 只改一处 — 选择不采用，因为三处逻辑必须一致，否则统计口径不统一。

### 决策 3: 空输出验证方式

**选择**: 编译后比较 `actual.trim() == ""`（与现有非空 case 逻辑一致）。

**替代方案**: 检查 `actual.is_empty()` — 选择不采用，因为编译产物可能含空白字符（换行符），trim 更稳健。

## Risks / Trade-offs

- **[Risk]** 某些空输出 case 可能因编译器 bug 实际产生非空输出 → 新增 FAIL → 需逐个修复
- **[Mitigation]** 先跑 `SPEC_STORE_CMD=run` 观察新增 FAIL 分布，评估修复成本
- **[Trade-off]** 测试时长增加 ~20-30 秒 → 可接受（文件 I/O 是瓶颈，编译本身很快）
- **[Trade-off]** 通过率数字会变化（分母变大）→ 更准确的测试覆盖度量
