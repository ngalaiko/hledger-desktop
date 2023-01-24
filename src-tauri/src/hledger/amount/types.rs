use std::fmt::Display;

use rust_decimal::Decimal;
use serde::Serialize;

#[derive(PartialEq, Eq)]
pub enum AmountSign {
    Plus,
    Minus,
}

/// Amounts used throughout
///
/// Defaults to displaying amounts in the format `<VALUE> <CURRENCY>`, using a dot as the decimal separator.
///
/// # Example:
///
/// ```
/// use rust_decimal_macros::dec;
/// use hledger_parse::Amount;
///
/// let amount = Amount { currency: "EUR".to_string(), value: dec!(19.99) };
/// assert_eq!("19.99 EUR", format!("{}", amount));
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct Amount {
    pub currency: String,
    pub value: Decimal,
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}
