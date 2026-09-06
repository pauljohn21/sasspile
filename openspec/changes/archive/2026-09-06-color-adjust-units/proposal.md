# Color Adjust Units

## Why

颜色 `adjust`/`change`/`scale` 函数的 percent 单位处理存在 bug：
- Oklch/Oklab adjust 对 unitless 值错误地除以 100（`$lightness: 0.5` unitless 被当作 0.005 而不是 0.5）
- Lab/Lch 缺乏 unit/percent 区分
- Modern RGB (display-p3/srgb/a98) 的 `%` 值被当作字面数值（`100%` → 100.0 而不是 1.0）
- Legacy RGB change-color 丢失了 alpha clamp

## What Changes

- 新增 `cie_channel` 提取器，区分数值单位：
  - 有单位 `%` → 解释为 channel max 的百分比（n/100 * max）
  - 无单位 n → 直接使用（在内部尺度）
  - `none` → NaN
- 新增 `apply_cie_channel` 辅助函数
- Oklch/Oklab L=1.0, C=0.4, a/b=0.4
- Lch/Lab L=100, C=150, a/b=125
- Modern RGB 统一使用 cie_channel(max=1.0)
- Legacy RGB change-color alpha clamp 恢复（+ NaN 保留 none）

## Impact

- sass-spec: 6426→6695 (+269) = 56.2%
- adjust: 78% (364/462, +27)
- change: 63% (242/379, +27)
- 影响文件：`color_adjust.rs`, `color_adjust_cie.rs`
