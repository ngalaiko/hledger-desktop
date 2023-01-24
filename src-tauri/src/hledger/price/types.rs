use std::fmt::Display;

use chrono::NaiveDate;
use serde::Serialize;

use crate::hledger::{amount::types::Amount, journal::types::Value, HLParserError};

/// Declared market prices
/// 
/// # Example:
/// 
/// ```
/// use chrono::NaiveDate;
/// use rust_decimal_macros::dec;
/// use hledger_parse::{Amount, Price};
/// 
/// let price = Price {
///     commodity: "EUR".to_string(),
///     date: NaiveDate::from_ymd(2022, 6, 23),
///     amount: Amount {
///         currency: "USD".to_string(),
///         value: dec!(1.05),
///     }
/// };
/// assert_eq!("P 2022-06-23 EUR 1.05 USD", format!("{}", price));
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct Price {
    pub commodity: String,
    pub date: NaiveDate,
    pub amount: Amount,
}

impl Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P {} {} {}", self.date, self.commodity, self.amount)
    }
}

impl TryInto<Price> for Value {
    type Error = HLParserError;

    fn try_into(self) -> Result<Price, Self::Error> {
        if let Value::Price(p) = self {
            Ok(p)
        } else {
            Err(HLParserError::Extract(self))
        }
    }
}
