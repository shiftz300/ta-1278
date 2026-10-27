//! Fixed native unit pairs and translated arithmetic. No rates or external services.
//! Evidence and C-library rounding limits: research archive: docs/evidence/converter-recovery.json.
mod data;
pub use data::{CATEGORIES, PAIRS};
pub const INPUT_LIMIT: usize = 12;

pub struct Conversion {
    pub title: &'static str,
    pub labels: [&'static str; 2],
    numerator: f64,
    denominator: f64,
    offsets: [f64; 2],
}
impl Conversion {
    pub fn convert(&self, text: &str, reverse: bool) -> Option<String> {
        let value: f64 = text.parse().ok()?;
        if !value.is_finite() {
            return None;
        }
        let result = if reverse {
            value * self.denominator / self.numerator + self.offsets[1]
        } else {
            value * self.numerator / self.denominator + self.offsets[0]
        };
        native_number(result)
    }
}
/// 88b9cf26..88b9d210: separate operations, ten decimals, bounded copy, zero trim.
fn native_number(value: f64) -> Option<String> {
    if !value.is_finite() || !(-9.9e100..=1e12).contains(&value) {
        return None;
    }
    if value.abs() < 1e-100 {
        return Some("0".into());
    }
    if value <= -1e13 || value.abs() <= 1e-11 {
        let formatted = format!("{value:.6e}");
        let (mantissa, exponent) = formatted.split_once('e')?;
        let exponent: i32 = exponent.parse().ok()?;
        return Some(format!("{mantissa}e{exponent:+03}"));
    }
    let mut text = format!("{value:.10}");
    text.truncate(if value < 0. { 16 } else { 12 });
    if text.contains('.') {
        text = text.trim_end_matches('0').trim_end_matches('.').into();
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_temperature_residue_and_reverse_offsets() {
        let f = &PAIRS[0][0];
        // Actual handset temperature-result.png. Do not replace with exact (F-32)*5/9.
        assert_eq!(f.convert("32", false).as_deref(), Some("-0.0000002222"));
        assert_eq!(f.convert("0", true).as_deref(), Some("32"));
        assert_eq!(PAIRS[0][1].convert("0", false).as_deref(), Some("-459.67"));
        assert_eq!(PAIRS[0][2].convert("273.15", false).as_deref(), Some("0"));
    }
    #[test]
    fn recovered_pairs_keep_rounded_factors_and_native_menu_order() {
        assert_eq!(
            PAIRS.iter().map(|p| p.len()).collect::<Vec<_>>(),
            [3, 5, 5, 5, 4]
        );
        assert_eq!(PAIRS[1][0].title, "磅-公斤");
        assert_eq!(PAIRS[1][0].convert("1", false).as_deref(), Some("0.4536"));
        assert_eq!(PAIRS[2][0].title, "英里-公里");
        assert_eq!(PAIRS[2][4].convert("1", false).as_deref(), Some("2.5401"));
        assert_eq!(PAIRS[4][3].convert("1", false).as_deref(), Some("3.78541"));
        for pair in PAIRS.iter().flat_map(|p| p.iter()) {
            assert!(pair.convert("1", false).is_some());
            assert!(pair.convert("1", true).is_some());
            assert!(pair.convert("", false).is_none());
            assert!(pair.convert("NaN", false).is_none());
        }
    }
    #[test]
    fn native_result_formatting_has_bounded_precision_and_exponents() {
        assert_eq!(
            native_number(123.4567891234).as_deref(),
            Some("123.45678912")
        );
        assert_eq!(native_number(1e-12).as_deref(), Some("1.000000e-12"));
        assert!(native_number(1e13).is_none());
        assert_eq!(native_number(-1e-110).as_deref(), Some("0"));
    }
}
