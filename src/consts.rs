//! 全局数值常量——消除散布在源码中的 magic number。
//!
//! 所有模块通过 `crate::consts::XXX` 引用，禁止内联数值字面量。

// ── RGB / 通道范围 ──

/// RGB 通道最大值（0-255 范围）。
pub const RGB_MAX: f64 = 255.0;

// ── 容差 / 精度 ──

/// Alpha 通道相等性比较容差。
pub const ALPHA_TOLERANCE: f64 = 0.0001;

/// 颜色 RGB 通道匹配容差（用于命名色反查）。
pub const COLOR_MATCH_TOLERANCE: f64 = 0.5;

/// 浮点精度截断因子（10 位小数，与 SCSS 规范一致）。
/// 用于除法：`v / FLOAT_PRECISION` 截断到 10 位小数。
/// 用于乘法：`v * FLOAT_PRECISION_INV` 放大后四舍五入。
pub const FLOAT_PRECISION: f64 = 1e-10;

/// 浮点精度截断逆因子（1 / 1e-10 = 1e10）。
/// 用于乘法放大：`(v * FLOAT_PRECISION_INV).round() / FLOAT_PRECISION_INV`。
pub const FLOAT_PRECISION_INV: f64 = 1e10;

/// 浮点噪声归零阈值（|v| < 此值视为 0）。
pub const FLOAT_NOISE_THRESHOLD: f64 = 1e-6;

/// 百分比归整阈值（接近 100 时归整）。
pub const PCT_ROUND_THRESHOLD: f64 = 1e-4;

// ── HSL / HWB ──

/// HSL/HWB hue 完整圆周角度。
pub const HUE_MAX: f64 = 360.0;

/// HSL 六色轮段边界（60° 一段）。
pub const HSL_SEGMENT: f64 = 60.0;

// ── Lab / Lch 常数 ──

/// Lab epsilon = 6³/29³ = 216/24389。
pub const LAB_EPSILON: f64 = 216.0 / 24389.0;

/// Lab kappa = 29³/3³ = 24389/27。
pub const LAB_KAPPA: f64 = 24389.0 / 27.0;

/// Lab L* 公式常数。
pub const LAB_L_SCALE: f64 = 116.0;

/// Lab L* 偏移常数。
pub const LAB_L_OFFSET: f64 = 16.0;

/// Lab a* 公式常数。
pub const LAB_A_SCALE: f64 = 500.0;

/// Lab b* 公式常数。
pub const LAB_B_SCALE: f64 = 200.0;

// ── D50 白点 ──

/// D50 白点 X 分量。
pub const D50_X: f64 = 0.3457 / 0.3585;

/// D50 白点 Y 分量。
pub const D50_Y: f64 = 1.0;

/// D50 白点 Z 分量。
pub const D50_Z: f64 = (1.0 - 0.3457 - 0.3585) / 0.3585;

// ── ProPhoto RGB ──

/// ProPhoto RGB 线性段阈值 = 1/512。
pub const PROPHOTO_ET: f64 = 1.0 / 512.0;

// ── sRGB 传递函数 ──

/// sRGB 线性段阈值上限。
pub const SRGB_LINEAR_THRESHOLD: f64 = 0.04045;

/// sRGB 线性段斜率。
pub const SRGB_LINEAR_SLOPE: f64 = 12.92;

/// sRGB 伽马传递函数偏移。
pub const SRGB_GAMMA_OFFSET: f64 = 0.055;

/// sRGB 伽马传递函数缩放。
pub const SRGB_GAMMA_SCALE: f64 = 1.055;

/// sRGB 伽马传递函数指数。
pub const SRGB_GAMMA_EXP: f64 = 2.4;

/// sRGB 逆线性段阈值上限。
pub const LINEAR_SRGB_THRESHOLD: f64 = 0.0031308;

/// sRGB 逆线性段斜率。
pub const LINEAR_SRGB_SLOPE: f64 = 12.92;

// ── A98 RGB ──

/// A98 RGB 伽马值 = 563/256。
pub const A98_GAMMA: f64 = 563.0 / 256.0;

/// A98 RGB 逆伽马值 = 256/563。
pub const A98_INV_GAMMA: f64 = 256.0 / 563.0;

// ── 百分比 ──

/// 百分比转换基数（0-1 → 0-100）。
pub const PCT_SCALE: f64 = 100.0;

/// 百分比单位字符串。
pub const PERCENT_UNIT: &str = "%";

/// 角度单位字符串。
pub const DEG_UNIT: &str = "deg";
