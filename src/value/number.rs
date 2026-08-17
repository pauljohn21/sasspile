//! Sass number type with unit support.

use std::fmt;
use crate::error::SassError;

/// A Sass number with optional unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    pub value: f64,
    pub unit: Option<String>,
}

impl Number {
    pub fn new(value: f64, unit: Option<String>) -> Self {
        Self { value, unit }
    }

    pub fn unitless(value: f64) -> Self {
        Self { value, unit: None }
    }

    pub fn is_unitless(&self) -> bool {
        self.unit.is_none()
    }

    /// Get the unit string, or empty if unitless.
    pub fn unit_str(&self) -> &str {
        self.unit.as_deref().unwrap_or("")
    }

    /// Check if two numbers have compatible units.
    /// In Sass, same units or both unitless are compatible.
    pub fn is_compatible_with(&self, other: &Number) -> bool {
        match (&self.unit, &other.unit) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }

    /// Add two numbers with unit inference.
    pub fn add(&self, other: &Number) -> Result<Number, SassError> {
        if self.unit.is_none() {
            return Ok(Number::new(self.value + other.value, other.unit.clone()));
        }
        if other.unit.is_none() || self.unit == other.unit {
            return Ok(Number::new(self.value + other.value, self.unit.clone()));
        }
        Err(SassError::eval(
            format!("Incompatible units: {} and {}", self.unit_str(), other.unit_str()),
            crate::error::SourcePos::default(),
        ))
    }

    pub fn sub(&self, other: &Number) -> Result<Number, SassError> {
        self.add(&Number::new(-other.value, other.unit.clone()))
    }

    pub fn mul(&self, other: &Number) -> Number {
        // Unit inference: one unit cancels if both have units
        let unit = match (&self.unit, &other.unit) {
            (None, _) => other.unit.clone(),
            (_, None) => self.unit.clone(),
            (Some(a), Some(b)) if a == b => None, // units cancel
            _ => self.unit.clone(),
        };
        Number::new(self.value * other.value, unit)
    }

    pub fn div(&self, other: &Number) -> Number {
        let unit = match (&self.unit, &other.unit) {
            (None, _) => None,
            (_, None) => self.unit.clone(),
            (Some(a), Some(b)) if a == b => None,
            _ => self.unit.clone(),
        };
        Number::new(self.value / other.value, unit)
    }

    pub fn modulo(&self, other: &Number) -> Result<Number, SassError> {
        if other.value == 0.0 {
            return Err(SassError::eval(
                "modulo by zero",
                crate::error::SourcePos::default(),
            ));
        }
        let unit = self.unit.clone().or(other.unit.clone());
        Ok(Number::new(self.value % other.value, unit))
    }

    pub fn negate(&self) -> Number {
        Number::new(-self.value, self.unit.clone())
    }

    pub fn cmp(&self, other: &Number) -> std::cmp::Ordering {
        self.value.partial_cmp(&other.value).unwrap_or(std::cmp::Ordering::Equal)
    }

    /// Format the number for CSS output.
    /// Integers are displayed without decimal point.
    pub fn to_css_string(&self) -> String {
        let v = self.value;
        let num_str = if v == v.trunc() && v.is_finite() && v.abs() < 1e15 {
            format!("{}", v as i64)
        } else {
            // Round to 10 decimal places, trim trailing zeros
            let s = format!("{:.10}", v);
            let s = s.trim_end_matches('0');
            let s = s.trim_end_matches('.');
            s.to_string()
        };
        if let Some(ref u) = self.unit {
            format!("{}{}", num_str, u)
        } else {
            num_str
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_css_string())
    }
}
