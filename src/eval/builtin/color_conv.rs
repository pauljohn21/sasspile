#![allow(
    clippy::unreadable_literal,
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::excessive_precision,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! f64 精度色彩空间转换算法。
//!
//! 基于 CSS Color 4 规范定义的数学公式，用 f64 实现以避免 f32 精度损失。
//! 矩阵系数使用 W3C 参考实现 (conversions.js) 的有理数分数形式和高精度小数。
//! 支持 sRGB ↔ Lab/Lch/Oklab/Oklch/XYZ 转换。
//!
//! 扩展色彩空间（Display P3/A98/ProPhoto/Rec2020）已拆分到 color_conv_spaces.rs，
//! 通过此处 `pub use` 保持向后兼容。

// 重新导出扩展色彩空间函数（从 color_conv_spaces.rs）
pub use super::color_conv_spaces::{
    a98_rgb_to_srgb, display_p3_to_srgb, prophoto_to_srgb, rec2020_to_srgb,
    srgb_to_a98_rgb, srgb_to_display_p3, srgb_to_prophoto, srgb_to_rec2020,
    xyz_d50_to_xyz_d65,
};

// ── sRGB ↔ Linear sRGB ──

/// sRGB 通道值 (0-1) → 线性 sRGB。
/// 扩展传递函数：负值在轴上反射后用幂函数。
pub fn srgb_to_linear(c: f64) -> f64 {
    let sign = match c < 0.0 {
        true => -1.0,
        false => 1.0,
    };
    let abs = c.abs();
    match abs <= 0.04045 {
        true => c / 12.92,
        false => sign * ((abs + 0.055) / 1.055).powf(2.4),
    }
}

/// 线性 sRGB → sRGB 通道值 (0-1)。
/// 扩展传递函数：负值在轴上反射后用幂函数。
pub fn linear_to_srgb(c: f64) -> f64 {
    let sign = match c < 0.0 {
        true => -1.0,
        false => 1.0,
    };
    let abs = c.abs();
    match abs > 0.0031308 {
        true => sign * (1.055 * abs.powf(1.0 / 2.4) - 0.055),
        false => 12.92 * c,
    }
}

// ── sRGB ↔ XYZ (D50) ──
// 使用 CSS Color 4 规范参考实现的精确有理数分数矩阵。
// 路径: sRGB → linear sRGB → XYZ D65 → (Bradford) → XYZ D50
// 其中 sRGB→XYZ D65 和 D65→D50 合并为一个复合矩阵。

/// sRGB (0-1) → XYZ D50。
/// 复合矩阵 = `D65_to_D50` × `lin_sRGB_to_XYZ（使用规范中有理数分数形式`。
pub fn srgb_to_xyz_d50(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);

    // lin_sRGB → XYZ D65 (有理数分数形式)
    let x65 = 506752.0 / 1228815.0 * rl + 87881.0 / 245763.0 * gl + 12673.0 / 70218.0 * bl;
    let y65 = 87098.0 / 409605.0 * rl + 175762.0 / 245763.0 * gl + 12673.0 / 175545.0 * bl;
    let z65 = 7918.0 / 409605.0 * rl + 87881.0 / 737289.0 * gl + 1001167.0 / 1053270.0 * bl;

    // Bradford D65 → D50
    let x = 1.0479297925449969 * x65 + 0.022946870601609652 * y65 - 0.05019226628920524 * z65;
    let y = 0.02962780877005599 * x65 + 0.9904344267538799 * y65 - 0.017073799063418826 * z65;
    let z = -0.009243040646204504 * x65 + 0.015055191490298152 * y65 + 0.7518742814281371 * z65;

    (x, y, z)
}

/// XYZ D50 → sRGB (0-1)。
/// 复合矩阵 = `XYZ_to_lin_sRGB` × `D50_to_D65（使用规范中有理数分数形式`。
pub fn xyz_d50_to_srgb(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    // Bradford D50 → D65
    let x65 = 0.955473421488075 * x - 0.02309845494876471 * y + 0.06325924320057072 * z;
    let y65 = -0.0283697093338637 * x + 1.0099953980813041 * y + 0.021041441191917323 * z;
    let z65 = 0.012314014864481998 * x - 0.020507649298898964 * y + 1.330365926242124 * z;

    // XYZ D65 → lin_sRGB (有理数分数形式)
    let rl = 12831.0 / 3959.0 * x65 - 329.0 / 214.0 * y65 - 1974.0 / 3959.0 * z65;
    let gl = -851781.0 / 878810.0 * x65 + 1648619.0 / 878810.0 * y65 + 36519.0 / 878810.0 * z65;
    let bl = 705.0 / 12673.0 * x65 - 2585.0 / 12673.0 * y65 + 705.0 / 667.0 * z65;

    (linear_to_srgb(rl), linear_to_srgb(gl), linear_to_srgb(bl))
}

// ── XYZ D50 ↔ Lab ──
// 使用 CSS Color 4 规范参考实现的精确常数。
// D50 白点定义: [0.3457/0.3585, 1.0, (1.0-0.3457-0.3585)/0.3585]
const D50_X: f64 = 0.3457 / 0.3585;
const D50_Y: f64 = 1.0;
const D50_Z: f64 = (1.0 - 0.3457 - 0.3585) / 0.3585;

/// Lab epsilon = 6^3/29^3 = 216/24389
const LAB_EPSILON: f64 = 216.0 / 24389.0;
/// Lab kappa = 29^3/3^3 = 24389/27
const LAB_KAPPA: f64 = 24389.0 / 27.0;

fn lab_f(t: f64) -> f64 {
    match t > LAB_EPSILON {
        true => t.cbrt(),
        false => (LAB_KAPPA * t + 16.0) / 116.0,
    }
}

fn lab_f_inv(t: f64) -> f64 {
    let t3 = t * t * t;
    match t3 > LAB_EPSILON {
        true => t3,
        false => (116.0 * t - 16.0) / LAB_KAPPA,
    }
}

/// XYZ D50 → Lab。
/// 使用 CSS Color 4 规范的精确 D50 白点定义。
fn xyz_d50_to_lab(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let x_r = x / D50_X;
    let y_r = y / D50_Y;
    let z_r = z / D50_Z;

    let fx = lab_f(x_r);
    let fy = lab_f(y_r);
    let fz = lab_f(z_r);

    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);

    (l, a, b)
}

/// Lab → XYZ D50。
fn lab_to_xyz_d50(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let fy = (l + 16.0) / 116.0;
    let fx = a / 500.0 + fy;
    let fz = fy - b / 200.0;

    let x = D50_X * lab_f_inv(fx);
    let y = D50_Y * lab_f_inv(fy);
    let z = D50_Z * lab_f_inv(fz);

    (x, y, z)
}

// ── Lab ↔ Lch ──

/// Lab → Lch。
fn lab_to_lch(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let c = (a * a + b * b).sqrt();
    let h = b.atan2(a).to_degrees();
    let h = match h < 0.0 {
        true => h + 360.0,
        false => h,
    };
    (l, c, h)
}

/// Lch → Lab。
fn lch_to_lab(l: f64, c: f64, h: f64) -> (f64, f64, f64) {
    let h_rad = h.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();
    (l, a, b)
}

// ── sRGB ↔ Oklab ──
// CSS Color 4 规范使用 XYZ D65 → LMS → Oklab 路径（而非直接线性sRGB→Oklab）。
// 矩阵系数使用规范参考实现 (conversions.js) 的高精度值。
// 参考: https://github.com/w3c/csswg-drafts/issues/6642#issuecomment-943521484

/// XYZ D65 → Oklab。
/// 路径: XYZ → LMS → cbrt(LMS) → Oklab
fn xyz_d65_to_oklab(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    // XYZ → LMS
    let l = 0.819_022_437_996_703 * x + 0.3619062600528904 * y - 0.1288737815209879 * z;
    let m = 0.0329836539323885 * x + 0.9292868615863434 * y + 0.0361446663506424 * z;
    let s = 0.0481771893596242 * x + 0.2642395317527308 * y + 0.6335478284694309 * z;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // LMS → Oklab
    let l_ok = 0.210_454_268_309_314 * l_ + 0.7936177747023054 * m_ - 0.0040720430116193 * s_;
    let a_ok = 1.9779985324311684 * l_ - 2.428_592_242_048_58 * m_ + 0.450_593_709_617_411 * s_;
    let b_ok = 0.0259040424655478 * l_ + 0.7827717124575296 * m_ - 0.8086757549230774 * s_;

    (l_ok, a_ok, b_ok)
}

/// Oklab → XYZ D65。
fn oklab_to_xyz_d65(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    // Oklab → LMS (non-linear)
    let l_ = 1.0000000000000000 * l + 0.3963377773761749 * a + 0.2158037573099136 * b;
    let m_ = 1.0000000000000000 * l - 0.1055613458156586 * a - 0.0638541728258133 * b;
    let s_ = 1.0000000000000000 * l - 0.0894841775298119 * a - 1.2914855480194092 * b;

    // cube
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    // LMS → XYZ
    let x = 1.2268798758459243 * l - 0.5578149944602171 * m + 0.2813910456659647 * s;
    let y = -0.0405757452148008 * l + 1.112_286_803_280_317 * m - 0.0717110580655164 * s;
    let z = -0.0763729366746601 * l - 0.4214933324022432 * m + 1.5869240198367816 * s;

    (x, y, z)
}

/// sRGB (0-1) → Oklab。路径: sRGB → linear sRGB → XYZ D65 → Oklab。
pub fn srgb_to_oklab(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = srgb_to_xyz_d65(r, g, b);
    xyz_d65_to_oklab(x, y, z)
}

/// Oklab → sRGB (0-1)。路径: Oklab → XYZ D65 → sRGB。
pub fn oklab_to_srgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = oklab_to_xyz_d65(l, a, b);
    xyz_d65_to_srgb(x, y, z)
}

// ── Oklab ↔ Oklch ──

/// Oklab → Oklch。
pub fn oklab_to_oklch(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let c = (a * a + b * b).sqrt();
    let h = b.atan2(a).to_degrees();
    let h = match h < 0.0 {
        true => h + 360.0,
        false => h,
    };
    (l, c, h)
}

/// Oklch → Oklab。
pub fn oklch_to_oklab(l: f64, c: f64, h: f64) -> (f64, f64, f64) {
    let h_rad = h.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();
    (l, a, b)
}

// ── 完整转换函数 ──

/// sRGB (0-1) → Lab。
pub fn srgb_to_lab(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = srgb_to_xyz_d50(r, g, b);
    xyz_d50_to_lab(x, y, z)
}

/// Lab → sRGB (0-1)。
pub fn lab_to_srgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = lab_to_xyz_d50(l, a, b);
    xyz_d50_to_srgb(x, y, z)
}

/// sRGB (0-1) → Lch。
pub fn srgb_to_lch(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (l, a, b) = srgb_to_lab(r, g, b);
    lab_to_lch(l, a, b)
}

/// Lch → sRGB (0-1)。
pub fn lch_to_srgb(l: f64, c: f64, h: f64) -> (f64, f64, f64) {
    let (l, a, b) = lch_to_lab(l, c, h);
    lab_to_srgb(l, a, b)
}

/// sRGB (0-1) → Oklch。
pub fn srgb_to_oklch(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (l, a, b) = srgb_to_oklab(r, g, b);
    oklab_to_oklch(l, a, b)
}

/// Oklch → sRGB (0-1)。
pub fn oklch_to_srgb(l: f64, c: f64, h: f64) -> (f64, f64, f64) {
    let (l, a, b) = oklch_to_oklab(l, c, h);
    oklab_to_srgb(l, a, b)
}

/// sRGB (0-1) → XYZ D65。
/// 使用 CSS Color 4 规范参考实现的有理数分数矩阵。
pub fn srgb_to_xyz_d65(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);
    let x = 506752.0 / 1228815.0 * rl + 87881.0 / 245763.0 * gl + 12673.0 / 70218.0 * bl;
    let y = 87098.0 / 409605.0 * rl + 175762.0 / 245763.0 * gl + 12673.0 / 175545.0 * bl;
    let z = 7918.0 / 409605.0 * rl + 87881.0 / 737289.0 * gl + 1001167.0 / 1053270.0 * bl;
    (x, y, z)
}

/// XYZ D65 → sRGB (0-1)。
/// 使用 CSS Color 4 规范参考实现的有理数分数矩阵。
pub fn xyz_d65_to_srgb(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let rl = 12831.0 / 3959.0 * x - 329.0 / 214.0 * y - 1974.0 / 3959.0 * z;
    let gl = -851781.0 / 878810.0 * x + 1648619.0 / 878810.0 * y + 36519.0 / 878810.0 * z;
    let bl = 705.0 / 12673.0 * x - 2585.0 / 12673.0 * y + 705.0 / 667.0 * z;
    (linear_to_srgb(rl), linear_to_srgb(gl), linear_to_srgb(bl))
}

/// sRGB (0-1) → 线性 sRGB (0-1)。
pub fn srgb_to_linear_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    (srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b))
}

/// 线性 sRGB (0-1) → sRGB (0-1)。
pub fn linear_srgb_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    (linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(b))
}

/// XYZ D65 → XYZ D50 (Bradford)。
pub fn xyz_d65_to_xyz_d50(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let x_d50 = 1.0479297925449969 * x + 0.022946870601609652 * y - 0.05019226628920524 * z;
    let y_d50 = 0.02962780877005599 * x + 0.9904344267538799 * y - 0.017073799063418826 * z;
    let z_d50 = -0.009243040646204504 * x + 0.015055191490298152 * y + 0.7518742814281371 * z;
    (x_d50, y_d50, z_d50)
}
