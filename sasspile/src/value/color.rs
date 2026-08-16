//! sRGB color representation with HSL support.

use std::fmt;

/// sRGB color with alpha channel.
#[derive(Debug, Clone)]
pub struct SassColor {
    /// Red channel (0-255).
    pub r: u8,
    /// Green channel (0-255).
    pub g: u8,
    /// Blue channel (0-255).
    pub b: u8,
    /// Alpha channel (0.0-1.0).
    pub a: f64,
}

impl SassColor {
    /// Create a new color.
    pub fn new(r: u8, g: u8, b: u8, a: f64) -> Self {
        Self {
            r,
            g,
            b,
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Create from #rrggbb hex value.
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as u8;
        let g = ((hex >> 8) & 0xFF) as u8;
        let b = (hex & 0xFF) as u8;
        Self::new(r, g, b, 1.0)
    }

    /// Create from #rgb shorthand hex.
    pub fn from_hex_short(hex: u32) -> Self {
        let r = ((hex >> 8) & 0xF) as u8;
        let g = ((hex >> 4) & 0xF) as u8;
        let b = (hex & 0xF) as u8;
        Self::new(r * 17, g * 17, b * 17, 1.0)
    }

    /// Get the alpha value.
    pub fn alpha(&self) -> f64 {
        self.a
    }

    /// Return true if the color is fully opaque.
    pub fn is_opaque(&self) -> bool {
        (self.a - 1.0).abs() < f64::EPSILON
    }

    /// Return true if the color is fully transparent.
    pub fn is_transparent(&self) -> bool {
        self.a.abs() < f64::EPSILON
    }

    /// Mix with another color (0.0 = self, 1.0 = other).
    pub fn mix(&self, other: &Self, amount: f64) -> Self {
        let t = amount.clamp(0.0, 1.0);
        let inv = 1.0 - t;
        Self {
            r: (self.r as f64 * inv + other.r as f64 * t).round() as u8,
            g: (self.g as f64 * inv + other.g as f64 * t).round() as u8,
            b: (self.b as f64 * inv + other.b as f64 * t).round() as u8,
            a: self.a * inv + other.a * t,
        }
    }

    // === HSL conversions ===

    /// Convert to HSL (h: 0-360, s: 0-1, l: 0-1).
    pub fn to_hsl(&self) -> (f64, f64, f64) {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        if (max - min).abs() < f64::EPSILON {
            return (0.0, 0.0, l);
        }

        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        let h = if (max - r).abs() < f64::EPSILON {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if (max - g).abs() < f64::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        } * 60.0;

        (h, s, l)
    }

    /// Create from HSL values.
    pub fn from_hsl(h: f64, s: f64, l: f64, a: f64) -> Self {
        let h = h.rem_euclid(360.0) / 360.0;
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);

        if s.abs() < f64::EPSILON {
            let v = (l * 255.0).round() as u8;
            return Self::new(v, v, v, a);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

        Self::new(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
            a,
        )
    }

    /// Get hue (0-360).
    pub fn hue(&self) -> f64 {
        self.to_hsl().0
    }

    /// Get saturation (0-1).
    pub fn saturation(&self) -> f64 {
        self.to_hsl().1
    }

    /// Get lightness (0-1).
    pub fn lightness(&self) -> f64 {
        self.to_hsl().2
    }

    /// Set hue, returning a new color.
    pub fn with_hue(&self, h: f64) -> Self {
        let (_, s, l) = self.to_hsl();
        Self::from_hsl(h, s, l, self.a)
    }

    /// Set saturation, returning a new color.
    pub fn with_saturation(&self, s: f64) -> Self {
        let (h, _, l) = self.to_hsl();
        Self::from_hsl(h, s, l, self.a)
    }

    /// Set lightness, returning a new color.
    pub fn with_lightness(&self, l: f64) -> Self {
        let (h, s, _) = self.to_hsl();
        Self::from_hsl(h, s, l, self.a)
    }

    /// Adjust hue by degrees.
    pub fn adjust_hue(&self, degrees: f64) -> Self {
        self.with_hue(self.hue() + degrees)
    }

    /// Lighten by percentage (0-100).
    pub fn lighten(&self, amount: f64) -> Self {
        let pct = amount.clamp(0.0, 100.0) / 100.0;
        let (_h, _s, l) = self.to_hsl();
        self.with_lightness(l + (1.0 - l) * pct)
    }

    /// Darken by percentage (0-100).
    pub fn darken(&self, amount: f64) -> Self {
        let pct = amount.clamp(0.0, 100.0) / 100.0;
        let (_h, _s, l) = self.to_hsl();
        self.with_lightness(l - l * pct)
    }

    /// Increase saturation by percentage (0-100).
    pub fn saturate(&self, amount: f64) -> Self {
        let pct = amount.clamp(0.0, 100.0) / 100.0;
        let (_h, s, _l) = self.to_hsl();
        self.with_saturation(s + (1.0 - s) * pct)
    }

    /// Decrease saturation by percentage (0-100).
    pub fn desaturate(&self, amount: f64) -> Self {
        let pct = amount.clamp(0.0, 100.0) / 100.0;
        let (_h, s, _l) = self.to_hsl();
        self.with_saturation(s - s * pct)
    }

    /// Convert to grayscale.
    pub fn grayscale(&self) -> Self {
        self.desaturate(100.0)
    }

    /// Invert the color.
    pub fn invert(&self) -> Self {
        Self::new(255 - self.r, 255 - self.g, 255 - self.b, self.a)
    }

    /// Set alpha channel.
    pub fn with_alpha(&self, alpha: f64) -> Self {
        Self::new(self.r, self.g, self.b, alpha.clamp(0.0, 1.0))
    }

    /// Get opacity (alpha).
    pub fn opacity(&self) -> f64 {
        self.a
    }

    /// Set opacity.
    pub fn with_opacity(&self, opacity: f64) -> Self {
        self.with_alpha(opacity.clamp(0.0, 1.0))
    }

    /// Fade in by amount (alias for opacity adjustment).
    pub fn fade_in(&self, amount: f64) -> Self {
        self.with_alpha(self.a + amount.clamp(0.0, 1.0))
    }

    /// Fade out by amount.
    pub fn fade_out(&self, amount: f64) -> Self {
        self.with_alpha(self.a - amount.clamp(0.0, 1.0))
    }

    /// Fade to alpha.
    pub fn fade(&self, alpha: f64) -> Self {
        self.with_alpha(alpha.clamp(0.0, 1.0))
    }

    /// Complement (rotate hue by 180).
    pub fn complement(&self) -> Self {
        self.adjust_hue(180.0)
    }

    /// Scale a channel by a percentage (-100 to 100).
    pub fn scale(&self, r_scale: f64, g_scale: f64, b_scale: f64, a_scale: f64) -> Self {
        Self::new(
            scale_channel(self.r, r_scale),
            scale_channel(self.g, g_scale),
            scale_channel(self.b, b_scale),
            (self.a + self.a * (a_scale / 100.0)).clamp(0.0, 1.0),
        )
    }

    /// Adjust a channel by a percentage (-100 to 100).
    pub fn adjust(&self, r_adj: f64, g_adj: f64, b_adj: f64, a_adj: f64) -> Self {
        Self::new(
            adjust_channel(self.r, r_adj),
            adjust_channel(self.g, g_adj),
            adjust_channel(self.b, b_adj),
            (self.a + a_adj / 100.0).clamp(0.0, 1.0),
        )
    }

    /// Get red channel value.
    pub fn red(&self) -> u8 {
        self.r
    }

    /// Get green channel value.
    pub fn green(&self) -> u8 {
        self.g
    }

    /// Get blue channel value.
    pub fn blue(&self) -> u8 {
        self.b
    }
}

/// Helper for HSL to RGB conversion.
fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// Scale a channel by percentage towards max/min.
fn scale_channel(c: u8, pct: f64) -> u8 {
    let pct = pct.clamp(-100.0, 100.0) / 100.0;
    if pct >= 0.0 {
        let diff = 255 - c;
        (c as f64 + diff as f64 * pct).round() as u8
    } else {
        (c as f64 + c as f64 * pct).round() as u8
    }
}

/// Adjust a channel by a percentage.
fn adjust_channel(c: u8, pct: f64) -> u8 {
    let pct = pct.clamp(-100.0, 100.0) / 100.0;
    if pct >= 0.0 {
        let diff = 255 - c;
        (c as f64 + diff as f64 * pct).round() as u8
    } else {
        (c as f64 + c as f64 * pct).round() as u8
    }
}

impl PartialEq for SassColor {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r
            && self.g == other.g
            && self.b == other.b
            && (self.a - other.a).abs() < 0.001
    }
}

impl fmt::Display for SassColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_opaque() {
            write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "rgba({}, {}, {}, {})",
                self.r, self.g, self.b, self.a
            )
        }
    }
}

/// Named CSS colors.
#[allow(dead_code)]
pub const NAMED_COLORS: &[(&str, u32)] = &[
    ("black", 0x000000),
    ("white", 0xFFFFFF),
    ("red", 0xFF0000),
    ("green", 0x008000),
    ("blue", 0x0000FF),
    ("transparent", 0x000000),
];
