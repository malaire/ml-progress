//! Internal types.
//!
//! These are not meant to be used directly
//! but need to be public for macros to work.

use std::fmt;

use crate::State;

// ======================================================================
// Item - PUBLIC

/// _Internal_ An item shown on progress indicator line.
pub enum Item {
    Fill(FillItem),
    Fn(Box<dyn Fn(&State) -> String + Send + Sync>),
    Literal(String),
}

// ======================================================================
// FillItem - PUBLIC

/// _Internal_ An item which fills remaining space on the line.
pub enum FillItem {
    Bar,
    Message,
}

// ======================================================================
// FormatFloat - PUBLIC

/// _Internal_ Wrapper for custom formatting of `f64`.
pub struct FormatFloat {
    value: f64,
    // This applies only if `#` flag is used.
    ignore_precision: bool,
}

impl FormatFloat {
    pub fn new(value: f64, ignore_precision: bool) -> Self {
        // I don't need negative values for now.
        assert!(value >= 0.0);

        Self {
            value,
            ignore_precision,
        }
    }
}

// ======================================================================
// FormatFloat - IMPL DISPLAY

impl fmt::Display for FormatFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            if self.ignore_precision {
                f.pad_integral(true, "", &format!("{:.0}", self.value))
            } else {
                let scale = if self.value < 10.0 {
                    0
                } else {
                    self.value.log10().floor() as usize
                };

                let fit_width = f.precision().unwrap_or(4);
                let precision = fit_width.saturating_sub(scale + 2);
                f.pad_integral(true, "", &format!("{:.*}", precision, self.value))
            }
        } else {
            self.value.fmt(f)
        }
    }
}

// ======================================================================
// FormatInteger - PUBLIC

/// _Internal_ Wrapper for custom formatting of `u64`.
pub struct FormatInteger<'a> {
    value: u64,
    thousands_separator: &'a str,
}

impl<'a> FormatInteger<'a> {
    pub fn new(value: u64, thousands_separator: &'a str) -> Self {
        Self {
            value,
            thousands_separator,
        }
    }
}

// ======================================================================
// FormatInteger - IMPL DISPLAY

impl<'a> fmt::Display for FormatInteger<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.pad_integral(
                true,
                "",
                &crate::group_digits(self.value, self.thousands_separator),
            )
        } else {
            self.value.fmt(f)
        }
    }
}

// ======================================================================
// FormatPrefix - PUBLIC

/// _Internal_ Wrapper for custom formatting of prefix.
pub struct FormatPrefix(&'static str);

impl FormatPrefix {
    pub fn new(value: &'static str) -> Self {
        Self(value)
    }
}

// ======================================================================
// FormatPrefix - IMPL DISPLAY

impl fmt::Display for FormatPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            if self.0.is_empty() {
                Ok(())
            } else {
                write!(f, " {}", self.0)
            }
        } else {
            self.0.fmt(f)
        }
    }
}

// ======================================================================
// TESTS

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // FormatFloat

    #[test]
    fn format_float() {
        assert_eq!(format!("{:#.4}", FormatFloat::new(1.23456, false)), "1.23");
        assert_eq!(format!("{:#.4}", FormatFloat::new(12.3456, false)), "12.3");
        assert_eq!(format!("{:#.4}", FormatFloat::new(123.456, false)), "123");
        assert_eq!(format!("{:#.4}", FormatFloat::new(1234.56, false)), "1235");
        assert_eq!(format!("{:#.4}", FormatFloat::new(12345.6, false)), "12346");
    }

    #[test]
    fn format_float_0() {
        assert_eq!(format!("{:#.4}", FormatFloat::new(0.0, false)), "0.00");
    }

    #[test]
    fn format_float_ignore_precision() {
        assert_eq!(format!("{:#.4}", FormatFloat::new(12.34, true)), "12");
    }
}
