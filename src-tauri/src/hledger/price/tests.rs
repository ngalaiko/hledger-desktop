use chrono::NaiveDate;
use rust_decimal_macros::dec;

use crate::hledger::amount::types::Amount;

use super::{parsers::parse_price, types::Price};

#[test]
fn test_valid_price() {
    assert_eq!(
        parse_price("P 2017-01-01 EUR SEK 9.552532877").unwrap(),
        (
            "",
            Price {
                commodity: "EUR".to_string(),
                date: NaiveDate::from_ymd(2017, 1, 1),
                amount: Amount {
                    currency: "SEK".to_string(),
                    value: dec!(9.552532877),
                }
            }
        )
    )
}
