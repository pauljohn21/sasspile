//! f64 精度色彩空间转换算法。
//!
//! 基于 CSS Color 4 规范定义的数学公式，用 f64 实现以避免 f32 精度损失。
//! 矩阵系数使用 W3C 参考实现 (conversions.js) 的有理数分数形式和高精度小数。
//! 支持 sRGB ↔ Lab/Lch/Oklab/Oklch/XYZ/DisplayP3/LinearRGB 转换。

// ── sRGB ↔ Linear sRGB ──

/// sRGB 通道值 (0-1) → 线性 sRGB。
/// 扩展传递函数：负值在轴上反射后用幂函数。
fn srgb_to_linear(c: f64) -> f64 {
    let sign = if c < 0.0 { -1.0 } else { 1.0 };
    let abs = c.abs();
    if abs <= 0.04045 {
        c / 12.92
    } else {
        sign * ((abs + 0.055) / 1.055).powf(2.4)
    }
}

/// 线性 sRGB → sRGB 通道值 (0-1)。
/// 扩展传递函数：负值在轴上反射后用幂函数。
fn linear_to_srgb(c: f64) -> f64 {
    let sign = if c < 0.0 { -1.0 } else { 1.0 };
    let abs = c.abs();
    if abs > 0.0031308 {
        sign * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
    } else {
        12.92 * c
    }
}

// ── sRGB ↔ XYZ (D50) ──
// 使用 CSS Color 4 规范参考实现的精确有理数分数矩阵。
// 路径: sRGB → linear sRGB → XYZ D65 → (Bradford) → XYZ D50
// 其中 sRGB→XYZ D65 和 D65→D50 合并为一个复合矩阵。

/// sRGB (0-1) → XYZ D50。
/// 复合矩阵 = D65_to_D50 × lin_sRGB_to_XYZ（使用规范中有理数分数形式）。
fn srgb_to_xyz_d50(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
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
/// 复合矩阵 = XYZ_to_lin_sRGB × D50_to_D65（使用规范中有理数分数形式）。
fn xyz_d50_to_srgb(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
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
    if t > LAB_EPSILON {
        t.cbrt()
    } else {
        (LAB_KAPPA * t + 16.0) / 116.0
    }
}

fn lab_f_inv(t: f64) -> f64 {
    let t3 = t * t * t;
    if t3 > LAB_EPSILON {
        t3
    } else {
        (116.0 * t - 16.0) / LAB_KAPPA
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
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
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
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
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
    let (x, y, z) = srgb_to_xyz_d65(r, g, b);
    // XYZ D65 → linear P3 (有理数分数)
    let r_p3 = 446124.0 / 178915.0 * x - 333277.0 / 357830.0 * y - 72051.0 / 178915.0 * z;
    let g_p3 = -14852.0 / 17905.0 * x + 63121.0 / 35810.0 * y + 423.0 / 17905.0 * z;
    let b_p3 = 11844.0 / 330415.0 * x - 50337.0 / 660830.0 * y + 316169.0 / 330415.0 * z;
    (linear_to_srgb(r_p3), linear_to_srgb(g_p3), linear_to_srgb(b_p3))
}

// ── A98 RGB ──
// 矩阵使用 CSS Color 4 规范参考实现的有理数分数形式。

/// A98 RGB (0-1) → sRGB (0-1)。
pub fn a98_rgb_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // A98 RGB gamma 563/256 → linear (扩展传递函数，支持负值)
    let to_lin = |c: f64| {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
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
    let (x, y, z) = srgb_to_xyz_d65(r, g, b);
    // XYZ D65 → linear A98 (有理数分数)
    let rl = 1829569.0 / 896150.0 * x - 506331.0 / 896150.0 * y - 308931.0 / 896150.0 * z;
    let gl = -851781.0 / 878810.0 * x + 1648619.0 / 878810.0 * y + 36519.0 / 878810.0 * z;
    let bl = 16779.0 / 1248040.0 * x - 147721.0 / 1248040.0 * y + 1266979.0 / 1248040.0 * z;
    // linear → A98 gamma 256/563 (扩展传递函数)
    let to_gam = |c: f64| {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
        sign * c.abs().powf(256.0 / 563.0)
    };
    (to_gam(rl), to_gam(gl), to_gam(bl))
}

// ── ProPhoto RGB ──
// 矩阵使用 CSS Color 4 规范参考实现的高精度小数。
// gamma 1.8，线性段阈值 Et = 1/512。

/// ProPhoto RGB (0-1) → sRGB (0-1)。
pub fn prophoto_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // ProPhoto gamma decode (Et2 = 16/512 = 1/32)
    fn prophoto_gamma_decode(c: f64) -> f64 {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
        let abs = c.abs();
        if abs <= 16.0 / 512.0 {
            c / 16.0
        } else {
            sign * abs.powf(1.0 / 1.8)
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

/// sRGB (0-1) → ProPhoto RGB (0-1)。
pub fn srgb_to_prophoto(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = srgb_to_xyz_d50(r, g, b);
    // XYZ D50 → ProPhoto (高精度小数)
    let rl = 1.345_786_881_647_158_3 * x - 0.25557208737979464 * y - 0.05110186497554526 * z;
    let gl = -0.544_630_705_124_901_9 * x + 1.508_247_742_845_146_8 * y + 0.02052744743642139 * z;
    let bl = 0.00000000000000000 * x + 0.00000000000000000 * y + 1.211_967_545_638_945_2 * z;
    // ProPhoto gamma encode (Et = 1/512)
    fn prophoto_gamma_encode(c: f64) -> f64 {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
        let abs = c.abs();
        if abs >= 1.0 / 512.0 {
            sign * abs.powf(1.0 / 1.8)
        } else {
            16.0 * c
        }
    }
    (prophoto_gamma_encode(rl), prophoto_gamma_encode(gl), prophoto_gamma_encode(bl))
}

// ── Rec2020 ──
// 矩阵使用 CSS Color 4 规范参考实现的有理数分数形式。
// gamma 2.4 (与 sRGB 相同的幂函数，但无线性段)。

/// Rec2020 (0-1) → sRGB (0-1)。
pub fn rec2020_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // Rec2020 gamma decode: pow(2.4) (扩展传递函数)
    fn rec2020_decode(c: f64) -> f64 {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
        sign * c.abs().powf(2.4)
    }
    let rl = rec2020_decode(r);
    let gl = rec2020_decode(g);
    let bl = rec2020_decode(b);
    // Rec2020 → XYZ D65 (有理数分数)
    let x = 63426534.0 / 99577255.0 * rl + 20160776.0 / 139408157.0 * gl + 47086771.0 / 278816314.0 * bl;
    let y = 26158966.0 / 99577255.0 * rl + 472592308.0 / 697040785.0 * gl + 8267143.0 / 139408157.0 * bl;
    let z = 0.0 * rl + 19567812.0 / 697040785.0 * gl + 295819943.0 / 278816314.0 * bl;
    xyz_d65_to_srgb(x, y, z)
}

/// sRGB (0-1) → Rec2020 (0-1)。
pub fn srgb_to_rec2020(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = srgb_to_xyz_d65(r, g, b);
    // XYZ D65 → linear Rec2020 (有理数分数)
    let rl = 30757411.0 / 17917100.0 * x - 6372589.0 / 17917100.0 * y - 4539589.0 / 17917100.0 * z;
    let gl = -19765991.0 / 29648200.0 * x + 47925759.0 / 29648200.0 * y + 467509.0 / 29648200.0 * z;
    let bl = 792561.0 / 44930125.0 * x - 1921689.0 / 44930125.0 * y + 42328811.0 / 44930125.0 * z;
    // linear → Rec2020 gamma: pow(1/2.4) (扩展传递函数)
    fn rec2020_encode(c: f64) -> f64 {
        let sign = if c < 0.0 { -1.0 } else { 1.0 };
        sign * c.abs().powf(1.0 / 2.4)
    }
    (rec2020_encode(rl), rec2020_encode(gl), rec2020_encode(bl))
}

// ── XYZ D50 ↔ XYZ D65 ──
// Bradford 色适应矩阵，使用 CSS Color 4 规范参考实现的高精度值。

/// XYZ D50 → XYZ D65 (Bradford)。
pub fn xyz_d50_to_xyz_d65(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let x_d65 = 0.955473421488075 * x - 0.02309845494876471 * y + 0.06325924320057072 * z;
    let y_d65 = -0.0283697093338637 * x + 1.0099953980813041 * y + 0.021041441191917323 * z;
    let z_d65 = 0.012314014864481998 * x - 0.020507649298898964 * y + 1.330365926242124 * z;
    (x_d65, y_d65, z_d65)
}

/// XYZ D65 → XYZ D50 (Bradford)。
pub fn xyz_d65_to_xyz_d50(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let x_d50 = 1.0479297925449969 * x + 0.022946870601609652 * y - 0.05019226628920524 * z;
    let y_d50 = 0.02962780877005599 * x + 0.9904344267538799 * y - 0.017073799063418826 * z;
    let z_d50 = -0.009243040646204504 * x + 0.015055191490298152 * y + 0.7518742814281371 * z;
    (x_d50, y_d50, z_d50)
}
