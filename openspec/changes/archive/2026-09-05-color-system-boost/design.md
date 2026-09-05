## Context

sasspile 颜色系统已有完整的 CSS Color 4 架构（ColorSpace enum、ColorOutput、f64 精度转换矩阵），但 sass-spec 颜色测试通过率仅 39%（2403/6027）。核心问题分布在 5 个领域：

1. **Legacy 同空间转换**：HWB→HWB 直接返回原值，spec 要求规范化为 HSL 输出
2. **color() 单参数透传**：`color(color(srgb ...))` 报错，应透传内部 Color 值
3. **none/missing 通道**：`hwb(none 20% 30%)` 等未被解析，spec 要求 missing 通道在转换时取默认值 0
4. **color.mix 现代空间**：仅支持 legacy RGB 混合，缺少 lab/oklab/oklch 等空间混合
5. **序列化精度**：小数位数和百分比格式与 spec 期望不完全对齐

## Goals / Non-Goals

**Goals:**
- 颜色 sass-spec 通过率从 39% 提升到 55%+
- 取消颜色目录跳过，颜色测试纳入全量统计
- 支持 CSS Color 4 `none` 关键字通道
- `color.mix` 支持现代颜色空间

**Non-Goals:**
- 不实现 `relative-color-syntax`（`from` 关键字）——这是独立特性
- 不实现 `.sass` 缩进语法
- 不追求 100% 颜色 spec 通过——部分失败涉及精确浮点比较，优先广覆盖

## Decisions

### D0: 每种颜色函数独立实现（核心架构决策）
**决策**：每种颜色操作函数（adjust/change/scale/mix/invert/grayscale/channel/to-space/to-gamut 等）MUST 有完全独立的实现函数，按颜色空间分派，禁止在一个大 match arm 中共享逻辑。
**理由**：历史多次失败证明，将所有颜色逻辑堆在一个大 match 中会导致：
1. 修改一个函数时意外影响其他函数（共享变量、共享分支逻辑）
2. 上下文丢失——大函数内部无法追踪具体哪个颜色空间的逻辑出问题
3. 矛盾代码——不同空间的需求矛盾时，后续修改覆盖先前逻辑
4. 抖动——AI 在长文件中反复修改同一函数，产生不一致实现
**架构**：每个颜色空间（legacy RGB/HSL/HWB + 现代 Lab/Lch/Oklab/Oklch/DisplayP3/sRGB/XYZ 等）的每个操作（adjust/change/scale）MUST 是独立函数，如 `adjust_oklch()`、`change_lab()`、`scale_hsl()`。分派通过 match 选择调用哪个独立函数，但函数体完全隔离。

### D1: Legacy 同空间转换规范化
**决策**：`convert_space()` 中 legacy 空间（RGB/HSL/HWB）同空间转换时不直接返回原值，而是经过规范化计算。
**理由**：SCSS 规范要求 legacy 颜色在 `to-space()` 中做规范化——HWB→HWB 会通过内部 HSL 中转计算，输出为 HSL 格式（因为 legacy 颜色不保留空间格式信息）。
**替代方案**：保持直接返回 → 但这导致大量 spec 测试失败。

### D2: color() 单参数透传
**决策**：`parse_color_space()` 在收到单个 `Value::Color` 参数时直接返回该颜色。
**理由**：`color()` 函数在 CSS 中既是构造函数也是颜色空间转换函数。当参数已经是 Color 时，应透传。

### D3: none 通道处理
**决策**：在 `Color` 结构体的 `channels` 中用 `f64::NAN` 表示 missing 通道。序列化时检测 NAN 输出 `none`，转换时 missing 通道取 0。
**理由**：NAN 天然传播，不需要额外字段。CSS Color 4 规范定义 missing 通道在计算时取 0，在输出时保留 `none` 标记。

### D4: color.mix 现代空间
**决策**：`color.mix()` 根据颜色空间选择独立混合算法——legacy 用 RGB 混合（独立函数），现代空间在对应空间中线性插值（每个空间独立函数）。
**理由**：SCSS 规范要求 `mix()` 在颜色的原始空间中混合。按 D0 决策，每个空间的混合逻辑独立实现。

### D5: 序列化精度
**决策**：使用 `format_num()` 统一控制小数位数（最多 10 位，去除尾随零），百分比同理。每个颜色空间的序列化独立处理。
**理由**：sass-spec 期望输出使用精确的浮点值，但去除多余的尾随零。

## Risks / Trade-offs

- [NAN 精度风险] → NaN 比较语义可能导致意外的相等性判断 → 在 `PartialEq for Color` 中检查 `is_nan()`
- [legacy 规范化可能破坏现有通过测试] → 逐步修复，每步运行测试验证
- [序列化精度对齐可能影响非颜色测试] → 只修改颜色相关序列化路径
