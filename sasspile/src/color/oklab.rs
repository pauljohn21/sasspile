//! Oklab and Oklch color space — perceptually uniform color models.
//!
//! Oklab is a perceptually uniform color space designed for:
//! - Smooth interpolation (color-mix)
//! - Perceptually accurate lightness control
//! - Gamut mapping
//!
//! Reference: <https://bottosson.github.io/posts/oklab/>

/// Oklab color: L (lightness 0-1), a (green-red), b (blue-yellow).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OklabColor {
    /// Lightness (0-1).
    pub l: f64,
    /// Green-red axis.
    pub a: f64,
    /// Blue-yellow axis.
    pub b: f64,
}

impl OklabColor {
    /// Create a new Oklab color.
    pub fn new(l: f64, a: f64, b: f64) -> Self {
        Self { l, a, b }
    }

    /// Convert to sRGB (returns [r, g, b] each 0-1, may be out of gamut).
    pub fn to_srgb(&self) -> [f64; 3] {
        let l_ = self.l + 0.3963377774 * self.a + 0.2158037573 * self.b;
        let m_ = self.l - 0.1055613458 * self.a - 0.0638541728 * self.b;
        let s_ = self.l - 0.0894841775 * self.a - 1.2914855480 * self.b;

        let l = l_.powi(3);
        let m = m_.powi(3);
        let s = s_.powi(3);

        [
            4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
        ]
    }

    /// Create from sRGB values (each 0-1).
    pub fn from_srgb(r: f64, g: f64, b: f64) -> Self {
        // Linearize sRGB.
        let r = linearize(r);
        let g = linearize(g);
        let b = linearize(b);

        let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        Self {
            l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
        }
    }

    /// Convert to Oklch (polar form: lightness, chroma, hue).
    pub fn to_oklch(&self) -> OklchColor {
        let c = (self.a * self.a + self.b * self.b).sqrt();
        let h = self.b.atan2(self.a).to_degrees();
        let h = if h < 0.0 { h + 360.0 } else { h };
        OklchColor::new(self.l, c, h)
    }
}

/// Oklch color: L (lightness), C (chroma), H (hue 0-360).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OklchColor {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

impl OklchColor {
    pub fn new(l: f64, c: f64, h: f64) -> Self {
        Self { l, c, h }
    }

    /// Convert to Oklab (Cartesian form).
    pub fn to_oklab(&self) -> OklabColor {
        let h_rad = self.h.to_radians();
        OklabColor::new(
            self.l,
            self.c * h_rad.cos(),
            self.c * h_rad.sin(),
        )
    }
}

/// Linearize an sRGB channel (0-1) to linear light.
fn linearize(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear light back to sRGB gamma-encoded.
fn delinearize(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Convert Oklab directly to sRGB, clamping to [0, 1].
pub fn oklab_to_srgb(l: f64, a: f64, b: f64) -> [f64; 3] {
    let oklab = OklabColor::new(l, a, b);
    let linear = oklab.to_srgb();
    let r = delinearize(linear[0].clamp(0.0, 1.0));
    let g = delinearize(linear[1].clamp(0.0, 1.0));
    let b = delinearize(linear[2].clamp(0.0, 1.0));
    [r, g, b]
}

/// Convert sRGB to Oklab.
pub fn srgb_to_oklab(r: u8, g: u8, b: u8) -> OklabColor {
    OklabColor::from_srgb(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oklab_roundtrip() {
        // Red should convert back close to original.
        let oklab = OklabColor::from_srgb(1.0, 0.0, 0.0);
        let srgb = oklab.to_srgb();
        assert!(srgb[0] > 0.9, "red channel should be near 1");
        assert!(srgb[1] < 0.1, "green channel should be near 0");
        assert!(srgb[2] < 0.1, "blue channel should be near 0");
    }

    #[test]
    fn oklab_white() {
        let oklab = OklabColor::from_srgb(1.0, 1.0, 1.0);
        assert!((oklab.l - 1.0).abs() < 0.01);
    }

    #[test]
    fn oklab_black() {
        let oklab = OklabColor::from_srgb(0.0, 0.0, 0.0);
        assert!(oklab.l.abs() < 0.01);
    }

    #[test]
    fn oklch_polar_conversion() {
        let oklab = OklabColor::new(0.5, 0.1, 0.2);
        let oklch = oklab.to_oklch();
        let round = oklch.to_oklab();
        assert!((oklab.l - round.l).abs() < 1e-10);
        assert!((oklab.a - round.a).abs() < 1e-10);
        assert!((oklab.b - round.b).abs() < 1e-10);
    }

    #[test]
    fn oklab_to_srgb_clamped() {
        // Out-of-gamut should be clamped.
        let rgb = oklab_to_srgb(1.5, 0.5, 0.5);
        for &c in &rgb {
            assert!(c >= 0.0 && c <= 1.0);
        }
    }
}
