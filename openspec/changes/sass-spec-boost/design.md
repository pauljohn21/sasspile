## Context

sasspile 当前 sass-spec 通过率 52.5%（6205/11824），颜色函数目录占全部失败的 61%。核心问题分布在四个区域：

1. **color/to_space**（1637 失败）：CSS Color 4 `color()` 函数解析和序列化不完整
2. **color/scale**（238 失败）：scale 通道边界计算偏差
3. **color/change**（228 失败）：现代色彩空间参数校验缺失
4. **color/adjust + mix + hsl**（477 失败）：输出格式和精度问题

现有颜色系统已支持 ColorFormat 枚举追踪颜色创建方式，但在序列化和跨空间转换中存在不完整路径。

## Goals / Non-Goals

**Goals:**
- 将 sass-spec 通过率从 52.5% 提升至 60%+
- 修复 color/to_space 的最大失败来源（目标 47%→65%）
- 完成 color-scale/change/adjust 的现代色彩空间支持
- 修复 color/mix/hsl 的输出格式保留

**Non-Goals:**
- 不重新设计 ColorFormat 枚举（已有架构足够）
- 不修改 Env_SCOPE 链架构
- 不实现新的 CSS at-rules（已在 sass-spec-completeness 完成）

## Decisions

### Decision 1: color() 函数解析扩展
**选择**：在现有 `ColorFormat` 枚举基础上扩展 `color()` 解析，而非新增独立类型。
**理由**：现有架构已支持 `Srgb`/`DisplayP3`/`Lab`/`Lch`/`Oklab`/`Oklch`/`XyzD65`/`XyzD50` 格式，只需在 `color_conv.rs` 中补充解析入口。
**备选**：新增独立的 ColorFunction 类型 — 过于冗余，增加序列化复杂度。

### Decision 2: scale 算法修正
**选择**：采用 sass-spec 标准公式 `new = current + direction * (|max - current| * percent/100)`。
**理由**：基于当前值与极值距离的比例，而非当前值的百分比。
**备选**：百分比基于当前值 — 不符合规范，通过率无法提升。

### Decision 3: 调试策略
**选择**：对失败抽样运行 `RUST_LOG=trace` 提取 span 证据链，定位根因后修复。
**理由**：遵循 AGENTS.md 强制 4 步调试协议，避免猜测。

## Risks / Trade-offs

- **风险 1**：color/to_space 涉及复杂的色域转换矩阵计算，精度难以对齐 → 缓解：使用 `color` crate v0.3 作为参考实现验证
- **风险 2**：修改 scale/change 可能影响已通过的 RGB 用例 → 缓解：每次修改后跑 `cargo test --test color_algorithm_test` 守护
- **风险 3**：单文件超过 500 行限制 → 缓解：color_adjust.rs 如需扩展，拆分为 `color_scale.rs`/`color_change.rs`/`color_adjust_ops.rs`

## Migration Plan

无迁移需求。本次变更为纯增量修复，不影响现有公开 API 或数据格式。
