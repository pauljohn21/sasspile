//! f64 精度色彩空间转换算法。
//!
//! 基于 CSS Color 4 规范定义的数学公式，用 f64 实现以避免 f32 精度损失。
//! 支持 sRGB ↔ Lab/Lch/Oklab/Oklch/XYZ/DisplayP3/LinearRGB 转换。

// ── sRGB ↔ Linear sRGB ──

/// sRGB 通道值 (0-1) → 线性 sRGB。
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// 线性 sRGB → sRGB 通道值 (0-1)。
fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// ── sRGB ↔ XYZ (D50) ──

/// sRGB (0-1) → XYZ D50。
/// 使用 CSS Color 4 规范的 sRGB→XYZ D50 复合矩阵。
fn srgb_to_xyz_d50(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);

    // sRGB (linear, D65) → XYZ D50 复合矩阵
    // (sRGB→XYZ D65 + Bradford D65→D50 合并)
    let x = 0.4360747 * rl + 0.3850649 * gl + 0.1430804 * bl;
    let y = 0.2225045 * rl + 0.7168786 * gl + 0.0606169 * bl;
    let z = 0.0139322 * rl + 0.0971045 * gl + 0.7141733 * bl;

    (x, y, z)
}

/// XYZ D50 → sRGB (0-1)。
/// 使用 CSS Color 4 规范的 XYZ D50→sRGB 复合矩阵。
fn xyz_d50_to_srgb(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    // XYZ D50 → sRGB (linear, D65) 复合矩阵
    // (Bradford D50→D65 + XYZ D65→sRGB 合并)
    let rl = 3.1338561 * x - 1.6168667 * y - 0.4906146 * z;
    let gl = -0.9787684 * x + 1.9161415 * y + 0.0334540 * z;
    let bl = 0.0719453 * x - 0.2289914 * y + 1.4052427 * z;

    (linear_to_srgb(rl), linear_to_srgb(gl), linear_to_srgb(bl))
}

// ── XYZ D50 ↔ Lab ──

fn lab_f(t: f64) -> f64 {
    let delta: f64 = 6.0 / 29.0;
    if t > delta.powi(3) {
        t.powf(1.0 / 3.0)
    } else {
        t / (3.0 * delta * delta) + 4.0 / 29.0
    }
}

fn lab_f_inv(t: f64) -> f64 {
    let delta: f64 = 6.0 / 29.0;
    if t > delta {
        t.powi(3)
    } else {
        3.0 * delta * delta * (t - 4.0 / 29.0)
    }
}

/// XYZ D50 → Lab。
/// 参考：CSS Color 4 规范。
fn xyz_d50_to_lab(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    // D50 参考白点
    let x_r = x / 0.96422;
    let y_r = y / 1.0;
    let z_r = z / 0.82521;

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

    let x = 0.96422 * lab_f_inv(fx);
    let y = 1.0 * lab_f_inv(fy);
    let z = 0.82521 * lab_f_inv(fz);

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

/// 线性 sRGB → Oklab。
fn linear_srgb_to_oklab(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    let l_ok = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let a_ok = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let b_ok = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    (l_ok, a_ok, b_ok)
}

/// Oklab → 线性 sRGB。
fn oklab_to_linear_srgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b_lin = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    (r, g, b_lin)
}

/// sRGB (0-1) → Oklab。
pub fn srgb_to_oklab(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);
    linear_srgb_to_oklab(rl, gl, bl)
}

/// Oklab → sRGB (0-1)。
pub fn oklab_to_srgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let (rl, gl, bl) = oklab_to_linear_srgb(l, a, b);
    (linear_to_srgb(rl), linear_to_srgb(gl), linear_to_srgb(bl))
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
pub fn srgb_to_xyz_d65(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);
    let x = 0.41239079926595934 * rl + 0.357584339383878 * gl + 0.1804807884018343 * bl;
    let y = 0.21263900587151027 * rl + 0.715168678767756 * gl + 0.07219231536073371 * bl;
    let z = 0.01933081871559182 * rl + 0.11919477979462598 * gl + 0.9505321522496607 * bl;
    (x, y, z)
}

/// XYZ D65 → sRGB (0-1)。
pub fn xyz_d65_to_srgb(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let rl = 3.2409699419045226 * x - 1.5373831775700935 * y - 0.49861076029305714 * z;
    let gl = -0.9692436362808796 * x + 1.8759675015077202 * y + 0.04155505740717559 * z;
    let bl = 0.05563007969699366 * x - 0.20397695888897652 * y + 1.0569715142428786 * z;
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

/// Display P3 (0-1) → sRGB (0-1)。
pub fn display_p3_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // P3 → linear P3 (P3 和 sRGB 共享相同的 gamma 曲线)
    let rl = srgb_to_linear(r);
    let gl = srgb_to_linear(g);
    let bl = srgb_to_linear(b);
    // linear P3 → XYZ D65
    let x = 0.4865709486482162 * rl + 0.2656676931690929 * gl + 0.1982172852343625 * bl;
    let y = 0.2289745640697488 * rl + 0.6917385218365064 * gl + 0.0792869140937449 * bl;
    let z = 0.0000000000000000 * rl + 0.0451133818589026 * gl + 1.0439443689009760 * bl;
    xyz_d65_to_srgb(x, y, z)
}

/// sRGB (0-1) → Display P3 (0-1)。
pub fn srgb_to_display_p3(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = srgb_to_xyz_d65(r, g, b);
    // XYZ D65 → linear P3
    let r_p3 = 2.493496911941425 * x - 0.9313836179191239 * y - 0.4027107844507168 * z;
    let g_p3 = -0.8294889695615747 * x + 1.7626640603183461 * y + 0.0236246858419436 * z;
    let b_p3 = 0.0358458302437845 * x - 0.0761723892680418 * y + 0.9568845360079346 * z;
    // linear P3 → P3 (P3 和 sRGB 共享相同的 gamma 曲线)
    (linear_to_srgb(r_p3), linear_to_srgb(g_p3), linear_to_srgb(b_p3))
}

// ── A98 RGB ──

/// A98 RGB (0-1) → sRGB (0-1)。
pub fn a98_rgb_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // A98 RGB gamma 1/2.2 → linear
    let rl = r.powf(563.0 / 256.0);
    let gl = g.powf(563.0 / 256.0);
    let bl = b.powf(563.0 / 256.0);
    // A98 → XYZ D65
    let x = 0.5766690429 * rl + 0.1855582379 * gl + 0.1882286462 * bl;
    let y = 0.2973449752 * rl + 0.6273635663 * gl + 0.0752902835 * bl;
    let z = 0.0270313614 * rl + 0.0706888525 * gl + 0.9913375368 * bl;
    xyz_d65_to_srgb(x, y, z)
}

/// sRGB (0-1) → A98 RGB (0-1)。
pub fn srgb_to_a98_rgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = srgb_to_xyz_d65(r, g, b);
    let rl = 1.962427438 * x - 0.610534324 * y - 0.341340176 * z;
    let gl = -0.978764678 * x + 1.816423363 * y + 0.162337723 * z;
    let bl = 0.028686965 * x - 0.042977346 * y + 1.005733684 * z;
    (rl.powf(256.0 / 563.0), gl.powf(256.0 / 563.0), bl.powf(256.0 / 563.0))
}

// ── ProPhoto RGB ──

/// ProPhoto RGB (0-1) → sRGB (0-1)。
pub fn prophoto_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    // ProPhoto gamma (1.8)
    fn prophoto_gamma_decode(c: f64) -> f64 {
        if c < 0.03125 { 0.0 } else {
            let v = c.powf(1.0 / 1.8);
            v
        }
    }
    let rl = prophoto_gamma_decode(r);
    let gl = prophoto_gamma_decode(g);
    let bl = prophoto_gamma_decode(b);
    // ProPhoto → XYZ D50
    let x = 0.7976749 * rl + 0.1351937 * gl + 0.0313435 * bl;
    let y = 0.2880132 * rl + 0.7117400 * gl + 0.0000851 * bl;
    let z = 0.0000000 * rl + 0.0000000 * gl + 0.8252100 * bl;
    // XYZ D50 → sRGB
    xyz_d50_to_srgb(x, y, z)
}

/// sRGB (0-1) → ProPhoto RGB (0-1)。
pub fn srgb_to_prophoto(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = srgb_to_xyz_d50(r, g, b);
    // XYZ D50 → ProPhoto
    let rl = 1.3457868816471585 * x - 0.2555720873797946 * y - 0.0511018649755453 * z;
    let gl = -0.5446307051249019 * x + 1.5082477388709226 * y + 0.0205377574441066 * z;
    let bl = 0.0000000 * x + 0.0000000 * y + 1.2118127545051852 * z;
    // ProPhoto gamma
    fn prophoto_gamma_encode(c: f64) -> f64 {
        if c < 0.0 { 0.0 } else {
            c.powf(1.8)
        }
    }
    (prophoto_gamma_encode(rl), prophoto_gamma_encode(gl), prophoto_gamma_encode(bl))
}

// ── Rec2020 ──

/// Rec2020 (0-1) → sRGB (0-1)。
pub fn rec2020_to_srgb(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    fn rec2020_decode(c: f64) -> f64 {
        let a = 0.7976608376771017;
        let b = 0.0333634376239715;
        if c < b {
            c / 4.5
        } else {
            ((c + b) / (1.0 + a)).powf(1.0 / 0.45)
        }
    }
    let rl = rec2020_decode(r);
    let gl = rec2020_decode(g);
    let bl = rec2020_decode(b);
    // Rec2020 → XYZ D65
    let x = 0.6369591 * rl + 0.1446175 * gl + 0.1688584 * bl;
    let y = 0.2627049 * rl + 0.6779884 * gl + 0.0593067 * bl;
    let z = 0.0000000 * rl + 0.0280731 * gl + 1.0609269 * bl;
    xyz_d65_to_srgb(x, y, z)
}

/// sRGB (0-1) → Rec2020 (0-1)。
pub fn srgb_to_rec2020(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let (x, y, z) = srgb_to_xyz_d65(r, g, b);
    let rl = 1.7166562 * x - 0.3556737 * y - 0.2533620 * z;
    let gl = -0.6666844 * x + 1.6164785 * y + 0.0157687 * z;
    let bl = 0.0176399 * x - 0.0427706 * y + 0.9424026 * z;
    fn rec2020_encode(c: f64) -> f64 {
        let a = 0.7976608376771017;
        let b = 0.0333634376239715;
        if c < 0.0 { 0.0 } else {
            let v = (1.0 + a) * c.powf(0.45) - b;
            v.max(0.0)
        }
    }
    (rec2020_encode(rl), rec2020_encode(gl), rec2020_encode(bl))
}

// ── XYZ D50 ↔ XYZ D65 ──

/// XYZ D50 → XYZ D65。
pub fn xyz_d50_to_xyz_d65(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let x_d65 = 0.9555766 * x - 0.0230393 * y + 0.0631636 * z;
    let y_d65 = -0.0282895 * x + 1.0099176 * y + 0.0210077 * z;
    let z_d65 = 0.0122982 * x - 0.0204830 * y + 1.3299098 * z;
    (x_d65, y_d65, z_d65)
}

/// XYZ D65 → XYZ D50。
pub fn xyz_d65_to_xyz_d50(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let x_d50 = 1.0478112 * x + 0.0228866 * y - 0.0501270 * z;
    let y_d50 = 0.0295424 * x + 0.9904844 * y - 0.0170491 * z;
    let z_d50 = -0.0092345 * x + 0.0150436 * y + 0.7521316 * z;
    (x_d50, y_d50, z_d50)
}
