//! Sass color type with multi-space support.

use std::fmt;

/// Supported color spaces.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorSpace {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Srgb,
    SrgbLinear,
    DisplayP3,
    DisplayP3Linear,
    A98Rgb,
    ProphotoRgb,
    Rec2020,
    Xyz,
    XyzD50,
}

impl ColorSpace {
    pub fn name(&self) -> &str {
        match self {
            ColorSpace::Rgb => "rgb",
            ColorSpace::Hsl => "hsl",
            ColorSpace::Hwb => "hwb",
            ColorSpace::Lab => "lab",
            ColorSpace::Lch => "lch",
            ColorSpace::Oklab => "oklab",
            ColorSpace::Oklch => "oklch",
            ColorSpace::Srgb => "srgb",
            ColorSpace::SrgbLinear => "srgb-linear",
            ColorSpace::DisplayP3 => "display-p3",
            ColorSpace::DisplayP3Linear => "display-p3-linear",
            ColorSpace::A98Rgb => "a98-rgb",
            ColorSpace::ProphotoRgb => "prophoto-rgb",
            ColorSpace::Rec2020 => "rec2020",
            ColorSpace::Xyz => "xyz",
            ColorSpace::XyzD50 => "xyz-d50",
        }
    }

    pub fn is_legacy(&self) -> bool {
        matches!(self, ColorSpace::Rgb | ColorSpace::Hsl | ColorSpace::Hwb)
    }
}

/// A Sass color value.
#[derive(Debug, Clone)]
pub struct Color {
    pub space: ColorSpace,
    pub channels: [f64; 4],
    pub legacy: bool,
}

impl Color {
    pub fn rgb(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            space: ColorSpace::Rgb,
            channels: [r, g, b, a],
            legacy: true,
        }
    }

    pub fn hsl(h: f64, s: f64, l: f64, a: f64) -> Self {
        Self {
            space: ColorSpace::Hsl,
            channels: [h, s, l, a],
            legacy: true,
        }
    }

    pub fn hwb(h: f64, w: f64, b: f64, a: f64) -> Self {
        Self {
            space: ColorSpace::Hwb,
            channels: [h, w, b, a],
            legacy: true,
        }
    }

    pub fn red(&self) -> f64 {
        match self.space {
            ColorSpace::Rgb => self.channels[0],
            _ => self.to_rgb().channels[0],
        }
    }

    pub fn green(&self) -> f64 {
        match self.space {
            ColorSpace::Rgb => self.channels[1],
            _ => self.to_rgb().channels[1],
        }
    }

    pub fn blue(&self) -> f64 {
        match self.space {
            ColorSpace::Rgb => self.channels[2],
            _ => self.to_rgb().channels[2],
        }
    }

    pub fn alpha(&self) -> f64 {
        self.channels[3]
    }

    pub fn hue(&self) -> f64 {
        match self.space {
            ColorSpace::Hsl => self.channels[0],
            ColorSpace::Hwb => self.channels[0],
            _ => self.to_hsl().channels[0],
        }
    }

    pub fn saturation(&self) -> f64 {
        match self.space {
            ColorSpace::Hsl => self.channels[1],
            _ => self.to_hsl().channels[1],
        }
    }

    pub fn lightness(&self) -> f64 {
        match self.space {
            ColorSpace::Hsl => self.channels[2],
            _ => self.to_hsl().channels[2],
        }
    }

    /// Convert to RGB color space.
    pub fn to_rgb(&self) -> Color {
        match self.space {
            ColorSpace::Rgb => self.clone(),
            ColorSpace::Hsl => Self::hsl_to_rgb(&self.channels),
            ColorSpace::Hwb => Self::hwb_to_rgb(&self.channels),
            _ => self.clone(),
        }
    }

    /// Convert to HSL color space.
    pub fn to_hsl(&self) -> Color {
        match self.space {
            ColorSpace::Hsl => self.clone(),
            ColorSpace::Rgb => Self::rgb_to_hsl(&self.channels),
            _ => self.clone(),
        }
    }

    fn hsl_to_rgb(c: &[f64; 4]) -> Color {
        let h = c[0] / 360.0;
        let s = c[1] / 100.0;
        let l = c[2] / 100.0;
        let a = c[3];

        if s == 0.0 {
            return Color::rgb(l * 255.0, l * 255.0, l * 255.0, a);
        }

        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        let hue2rgb = |t: f64| -> f64 {
            let mut t = t;
            if t < 0.0 { t += 1.0; }
            if t > 1.0 { t -= 1.0; }
            if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
            if t < 1.0 / 2.0 { return q; }
            if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
            p
        };
        let r = hue2rgb(h + 1.0 / 3.0) * 255.0;
        let g = hue2rgb(h) * 255.0;
        let b = hue2rgb(h - 1.0 / 3.0) * 255.0;
        Color::rgb(r, g, b, a)
    }

    fn rgb_to_hsl(c: &[f64; 4]) -> Color {
        let r = c[0] / 255.0;
        let g = c[1] / 255.0;
        let b = c[2] / 255.0;
        let a = c[3];
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < 1e-10 {
            return Color::hsl(0.0, 0.0, l * 100.0, a);
        }
        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        let h = if max == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        Color::hsl(h * 60.0, s * 100.0, l * 100.0, a)
    }

    fn hwb_to_rgb(c: &[f64; 4]) -> Color {
        let h = c[0];
        let w = c[1] / 100.0;
        let b = c[2] / 100.0;
        let a = c[3];
        if w + b >= 1.0 {
            let gray = w / (w + b) * 255.0;
            return Color::rgb(gray, gray, gray, a);
        }
        let hsl = Self::hsl_to_rgb(&[h, 100.0, 50.0, a]);
        let r = hsl.channels[0] / 255.0;
        let g = hsl.channels[1] / 255.0;
        let bl = hsl.channels[2] / 255.0;
        let r = r * (1.0 - w - b) + w;
        let g = g * (1.0 - w - b) + w;
        let bl = bl * (1.0 - w - b) + b;
        Color::rgb(r * 255.0, g * 255.0, bl * 255.0, a)
    }

    /// Format as hex string (#rgb or #rrggbb).
    pub fn to_hex(&self) -> String {
        let rgb = self.to_rgb();
        let r = (rgb.channels[0].round() as u8).min(255).max(0);
        let g = (rgb.channels[1].round() as u8).min(255).max(0);
        let b = (rgb.channels[2].round() as u8).min(255).max(0);
        if r % 17 == 0 && g % 17 == 0 && b % 17 == 0 {
            format!("#{:x}{:x}{:x}", r / 17, g / 17, b / 17)
        } else {
            format!("#{:02x}{:02x}{:02x}", r, g, b)
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.legacy {
            let rgb = self.to_rgb();
            let r = rgb.channels[0].round() as u8;
            let g = rgb.channels[1].round() as u8;
            let b = rgb.channels[2].round() as u8;
            let a = rgb.channels[3];
            if (a - 1.0).abs() < 1e-10 {
                write!(f, "#{:02x}{:02x}{:02x}", r, g, b)
            } else {
                write!(f, "rgba({}, {}, {}, {})", r, g, b, a)
            }
        } else {
            let parts: Vec<String> = self.channels.iter().map(|c| c.to_string()).collect();
            write!(f, "{}({})", self.space.name(), parts.join(", "))
        }
    }
}
