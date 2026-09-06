#![allow(
    clippy::unreadable_literal,
    clippy::many_single_char_names,
    clippy::single_char_pattern,
    clippy::excessive_precision,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
//! CSS Color 4 扩展色彩空间转换函数。
//!
//! 从 `color_conv.rs` 拆分，包含：
//! - Display P3 ↔ sRGB
//! - A98 RGB ↔ sRGB
//! - ProPhoto RGB ↔ sRGB
//! - Rec2020 ↔ sRGB
//! - XYZ D50 ↔ XYZ D65 (Bradford)

use super::color_conv::{
    linear_to_srgb, srgb_to_linear, xyz_d50_to_srgb, xyz_d65_to_srgb,
};

// ── Display P3 ──
// Display P3 和 sRGB 有相同的 gamma 曲线，只是原色域不同。
// 矩阵使用 CSS Color 4 规范参考实现的有理数分数形式。

/// Display P3 (0-1) → sRGB (0-1)。
pub fn display_p3_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);
    // linear P3 → XYZ D65 (有理数分数)
    let x = 608311.0 / 1250200.0 * rl + 189793.0 / 714400.0 * gl + 198249.0 / 1000160.0 * bl;
    let y = 35783.0 / 156275.0 * rl + 247089.0 / 357200.0 * gl + 198249.0 / 2500400.0 * bl;
    let z = 0.0 * rl + 32229.0 / 714400.0 * gl + 5220557.0 / 5000800.0 * bl;
    xyz_d65_to_srgb(x, y, z)
}

/// sRGB (0-1) → Display P3 (0-1)。
pub fn srgb_to_display_p3(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = super::color_conv::srgb_to_xyz_d65(r, g, b);
    // XYZ D65 → linear P3 (有理数分数)
    let r_p3 = 446124.0 / 178915.0 * x - 333277.0 / 357830.0 * y - 72051.0 / 178915.0 * z;
    let g_p3 = -14852.0 / 17905.0 * x + 63121.0 / 35810.0 * y + 423.0 / 17905.0 * z;
    let b_p3 = 11844.0 / 330415.0 * x - 50337.0 / 660830.0 * y + 316169.0 / 330415.0 * z;
    (
        linear_to_srgb(r_p3),
        linear_to_srgb(g_p3),
        linear_to_srgb(b_p3),
    )
}

// ── A98 RGB ──
// 矩阵使用 CSS Color 4 规范参考实现的有理数分数形式。

/// A98 RGB (0-1) → sRGB (0-1)。
pub fn a98_rgb_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // A98 RGB gamma 563/256 → linear (扩展传递函数，支持负值)
    let to_lin = |c: f64| {
        let sign = match c < 0.0 {
            true => -1.0,
            false => 1.0,
        };
        sign * c.abs().powf(563.0 / 256.0)
    };
    let rl = to_lin(r);
    let gl = to_lin(g);
    let bl = to_lin(b);
    // A98 → XYZ D65 (有理数分数)
    let x = 573536.0 / 994567.0 * rl + 263643.0 / 1420810.0 * gl + 187206.0 / 994567.0 * bl;
    let y = 591459.0 / 1989134.0 * rl + 6239551.0 / 9945670.0 * gl + 374412.0 / 4972835.0 * bl;
    let z = 53769.0 / 1989134.0 * rl + 351524.0 / 4972835.0 * gl + 4929758.0 / 4972835.0 * bl;
    xyz_d65_to_srgb(x, y, z)
}

/// sRGB (0-1) → A98 RGB (0-1)。
pub fn srgb_to_a98_rgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = super::color_conv::srgb_to_xyz_d65(r, g, b);
    // XYZ D65 → linear A98 (有理数分数)
    let rl = 1829569.0 / 896150.0 * x - 506331.0 / 896150.0 * y - 308931.0 / 896150.0 * z;
    let gl = -851781.0 / 878810.0 * x + 1648619.0 / 878810.0 * y + 36519.0 / 878810.0 * z;
    let bl = 16779.0 / 1248040.0 * x - 147721.0 / 1248040.0 * y + 1266979.0 / 1248040.0 * z;
    // linear → A98 gamma 256/563 (扩展传递函数)
    let to_gam = |c: f64| {
        let sign = match c < 0.0 {
            true => -1.0,
            false => 1.0,
        };
        sign * c.abs().powf(256.0 / 563.0)
    };
    (to_gam(rl), to_gam(gl), to_gam(bl))
}

// ── ProPhoto RGB ──
// 矩阵使用 CSS Color 4 规范参考实现的高精度小数。
// gamma 1.8，线性段阈值 Et = 1/512。

/// `ProPhoto` RGB (0-1) → sRGB (0-1)。
pub fn prophoto_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // ProPhoto gamma decode (Et2 = 16/512 = 1/32)
    fn prophoto_gamma_decode(c: f64) -> f64 {
        let sign = match c < 0.0 {
            true => -1.0,
            false => 1.0,
        };
        let abs = c.abs();
        match abs <= 16.0 / 512.0 {
            true => c / 16.0,
            false => sign * abs.powf(1.0 / 1.8),
        }
    }
    let rl = prophoto_gamma_decode(r);
    let gl = prophoto_gamma_decode(g);
    let bl = prophoto_gamma_decode(b);
    // ProPhoto → XYZ D50 (高精度小数)
    let x = 0.797_766_644_900_642_3 * rl + 0.13518129740053308 * gl + 0.031_347_734_128_392_2 * bl;
    let y = 0.288_074_828_819_401_3 * rl + 0.711_835_234_241_873 * gl + 0.00008993693872564 * bl;
    let z = 0.00000000000000000 * rl + 0.00000000000000000 * gl + 0.825_104_602_510_460_2 * bl;
    // XYZ D50 → sRGB
    xyz_d50_to_srgb(x, y, z)
}

/// `ProPhoto` gamma encode (Et = 1/512)。
fn prophoto_gamma_encode(c: f64) -> f64 {
    let sign = match c < 0.0 {
        true => -1.0,
        false => 1.0,
    };
    let abs = c.abs();
    match abs >= 1.0 / 512.0 {
        true => sign * abs.powf(1.0 / 1.8),
        false => 16.0 * c,
    }
}

/// sRGB (0-1) → `ProPhoto` RGB (0-1)。
pub fn srgb_to_prophoto(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = super::color_conv::srgb_to_xyz_d50(r, g, b);
    // XYZ D50 → ProPhoto (高精度小数)
    let rl = 1.345_786_881_647_158_3 * x - 0.25557208737979464 * y - 0.05110186497554526 * z;
    let gl = -0.544_630_705_124_901_9 * x + 1.508_247_742_845_146_8 * y + 0.02052744743642139 * z;
    let bl = 0.00000000000000000 * x + 0.00000000000000000 * y + 1.211_967_545_638_945_2 * z;
    // ProPhoto gamma encode (Et = 1/512)
    (
        prophoto_gamma_encode(rl),
        prophoto_gamma_encode(gl),
        prophoto_gamma_encode(bl),
    )
}

// ── Rec2020 ──
// 矩阵使用 CSS Color 4 规范参考实现的有理数分数形式。
// gamma 2.4 (与 sRGB 相同的幂函数，但无线性段)。

/// Rec2020 gamma encode: pow(1/2.4)。
fn rec2020_encode(c: f64) -> f64 {
    let sign = match c < 0.0 {
        true => -1.0,
        false => 1.0,
    };
    sign * c.abs().powf(1.0 / 2.4)
}

/// Rec2020 (0-1) → sRGB (0-1)。
pub fn rec2020_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // Rec2020 gamma decode: pow(2.4) (扩展传递函数)
    fn rec2020_decode(c: f64) -> f64 {
        let sign = match c < 0.0 {
            true => -1.0,
            false => 1.0,
        };
        sign * c.abs().powf(2.4)
    }
    let rl = rec2020_decode(r);
    let gl = rec2020_decode(g);
    let bl = rec2020_decode(b);
    // Rec2020 → XYZ D65 (有理数分数)
    let x = 63426534.0 / 99577255.0 * rl
        + 20160776.0 / 139408157.0 * gl
        + 47086771.0 / 278816314.0 * bl;
    let y = 26158966.0 / 99577255.0 * rl
        + 472592308.0 / 697040785.0 * gl
        + 8267143.0 / 139408157.0 * bl;
    let z = 0.0 * rl + 19567812.0 / 697040785.0 * gl + 295819943.0 / 278816314.0 * bl;
    xyz_d65_to_srgb(x, y, z)
}

/// sRGB (0-1) → Rec2020 (0-1)。
pub fn srgb_to_rec2020(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = super::color_conv::srgb_to_xyz_d65(r, g, b);
    // XYZ D65 → linear Rec2020 (有理数分数)
    let rl = 30757411.0 / 17917100.0 * x - 6372589.0 / 17917100.0 * y - 4539589.0 / 17917100.0 * z;
    let gl = -19765991.0 / 29648200.0 * x + 47925759.0 / 29648200.0 * y + 467509.0 / 29648200.0 * z;
    let bl = 792561.0 / 44930125.0 * x - 1921689.0 / 44930125.0 * y + 42328811.0 / 44930125.0 * z;
    // linear → Rec2020 gamma: pow(1/2.4) (扩展传递函数)
    (rec2020_encode(rl), rec2020_encode(gl), rec2020_encode(bl))
}

/// XYZ D50 → XYZ D65 (Bradford)。
pub fn xyz_d50_to_xyz_d65(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let x_d65 = 0.955473421488075 * x - 0.02309845494876471 * y + 0.06325924320057072 * z;
    let y_d65 = -0.0283697093338637 * x + 1.0099953980813041 * y + 0.021041441191917323 * z;
    let z_d65 = 0.012314014864481998 * x - 0.020507649298898964 * y + 1.330365926242124 * z;
    (x_d65, y_d65, z_d65)
}
