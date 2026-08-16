//! Numeric value with optional unit.

use std::fmt;

/// Number value with optional compound unit.
#[derive(Debug, Clone)]
pub struct Number {
    /// Magnitude.
    pub value: f64,
    /// Optional unit.
    pub unit: Unit,
}

/// CSS unit types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Unit {
    /// Unitless.
    None,
    /// Length units.
    Em,
    Rem,
    Px,
    Pt,
    Pc,
    In,
    Cm,
    Mm,
    Q,
    /// Angle units.
    Deg,
    Rad,
    Grad,
    Turn,
    /// Time units.
    S,
    Ms,
    /// Frequency units.
    Hz,
    Khz,
    /// Resolution units.
    Dpi,
    Dpcm,
    Dppx,
    /// Percentage.
    Percent,
    /// Compound unit (e.g., px/px).
    Compound(Vec<Unit>),
}

impl Unit {
    /// Parse unit from string.
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "" => Some(Unit::None),
            "em" => Some(Unit::Em),
            "rem" => Some(Unit::Rem),
            "px" => Some(Unit::Px),
            "pt" => Some(Unit::Pt),
            "pc" => Some(Unit::Pc),
            "in" => Some(Unit::In),
            "cm" => Some(Unit::Cm),
            "mm" => Some(Unit::Mm),
            "q" => Some(Unit::Q),
            "deg" => Some(Unit::Deg),
            "rad" => Some(Unit::Rad),
            "grad" => Some(Unit::Grad),
            "turn" => Some(Unit::Turn),
            "s" => Some(Unit::S),
            "ms" => Some(Unit::Ms),
            "hz" => Some(Unit::Hz),
            "khz" => Some(Unit::Khz),
            "dpi" => Some(Unit::Dpi),
            "dpcm" => Some(Unit::Dpcm),
            "dppx" => Some(Unit::Dppx),
            "%" => Some(Unit::Percent),
            _ => None,
        }
    }

    /// Return true if unit is compatible with another.
    pub fn is_compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (Unit::None, _) | (_, Unit::None) => true,
            (Unit::Em, Unit::Rem)
            | (Unit::Rem, Unit::Em)
            | (Unit::Px, Unit::Pt)
            | (Unit::Pt, Unit::Px) => true,
            _ => self == other,
        }
    }
}

impl Number {
    /// Create a new number value.
    pub fn new(value: f64, unit: Unit) -> Self {
        Self { value, unit }
    }

    /// Unitless number.
    pub fn unitless(value: f64) -> Self {
        Self {
            value,
            unit: Unit::None,
        }
    }

    /// Pixel value.
    pub fn px(value: f64) -> Self {
        Self {
            value,
            unit: Unit::Px,
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        (self.value - other.value).abs() < f64::EPSILON && self.unit.is_compatible(&other.unit)
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value.fract() == 0.0 {
            write!(f, "{}", self.value as i64)
        } else {
            write!(f, "{}", self.value)
        }
    }
}
