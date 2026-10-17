//! Display conversions for budgets.
//!
//! The save stores budgets as a raw `u32` in units of €100. Other currencies use fixed
//! approximate rates and are for display only — the save is always written in raw units.

/// Display multiplier between a raw value and the amount in euros.
pub const RAW_TO_EUR: u64 = 100;

/// Display currencies, with fixed approximate exchange rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Currency {
    #[default]
    Euro,
    UsDollar,
    Yen,
    Pound,
    BrazilianReal,
}

impl Currency {
    pub const ALL: [Currency; 5] = [
        Currency::Euro,
        Currency::UsDollar,
        Currency::Yen,
        Currency::Pound,
        Currency::BrazilianReal,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Currency::Euro => "€ Euro",
            Currency::UsDollar => "$ US Dollar",
            Currency::Yen => "¥ Yen",
            Currency::Pound => "£ Pound",
            Currency::BrazilianReal => "R$ Real",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Currency::Euro => "€",
            Currency::UsDollar => "$",
            Currency::Yen => "¥",
            Currency::Pound => "£",
            Currency::BrazilianReal => "R$",
        }
    }

    /// Units of this currency per euro (fixed approximation).
    pub fn per_euro(self) -> f64 {
        match self {
            Currency::Euro => 1.0,
            Currency::UsDollar => 1.1,
            Currency::Yen => 125.0,
            Currency::Pound => 0.9,
            Currency::BrazilianReal => 4.3,
        }
    }

    /// Amount shown for a raw stored value.
    pub fn display(self, raw: u32) -> u64 {
        let eur = u64::from(raw) * RAW_TO_EUR;
        (eur as f64 * self.per_euro()).round() as u64
    }

    /// Raw value for an amount typed in this currency (rounded to the nearest unit).
    /// Returns `None` if the result does not fit in a `u32`.
    pub fn to_raw(self, amount: u64) -> Option<u32> {
        let raw = (amount as f64 / self.per_euro() / RAW_TO_EUR as f64).round();
        if raw.is_finite() && raw >= 0.0 && raw <= f64::from(u32::MAX) {
            Some(raw as u32)
        } else {
            None
        }
    }
}

/// Parses a non-negative integer typed by a user, ignoring thousands separators
/// (`.`, `,`, `_`, `'` and spaces). Amounts are always whole numbers.
pub fn parse_amount(s: &str) -> Option<u64> {
    let digits: String = s
        .chars()
        .filter(|c| !matches!(c, '.' | ',' | '_' | ' ' | '\'' | '\u{a0}'))
        .collect();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Formats an integer with thousands separators, e.g. `8.498.197.200`.
pub fn group_thousands(value: u64, separator: char) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(separator);
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euro_is_raw_times_100() {
        assert_eq!(Currency::Euro.display(84_981_972), 8_498_197_200);
        assert_eq!(Currency::Euro.to_raw(8_498_197_200), Some(84_981_972));
        assert_eq!(Currency::Euro.display(99_999_999), 9_999_999_900);
    }

    #[test]
    fn currency_round_trip() {
        for c in Currency::ALL {
            for raw in [0u32, 1, 1234, 99_999_999] {
                assert_eq!(c.to_raw(c.display(raw)), Some(raw), "{c:?} {raw}");
            }
        }
        assert_eq!(Currency::Euro.to_raw(u64::MAX), None);
    }

    #[test]
    fn parsing() {
        assert_eq!(parse_amount("9.999.999.900"), Some(9_999_999_900));
        assert_eq!(parse_amount(" 1,000 "), Some(1000));
        assert_eq!(parse_amount("-5"), None);
        assert_eq!(parse_amount("1e5"), None);
        assert_eq!(parse_amount(""), None);
        assert_eq!(parse_amount("99999999999999999999999"), None);
    }

    #[test]
    fn grouping() {
        assert_eq!(group_thousands(0, '.'), "0");
        assert_eq!(group_thousands(999, '.'), "999");
        assert_eq!(group_thousands(1000, '.'), "1.000");
        assert_eq!(group_thousands(8_498_197_200, ','), "8,498,197,200");
    }
}
