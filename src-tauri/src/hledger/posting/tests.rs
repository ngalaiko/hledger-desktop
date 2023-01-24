use nom::error::ErrorKind;
use rust_decimal_macros::dec;

use crate::hledger::{amount::types::Amount, status::types::Status, HLParserError};

use super::{parsers::parse_posting, types::Posting};

#[test]
fn test_parse_simple_posting() {
    assert_eq!(
        parse_posting(" assets:cash  $100").unwrap(),
        (
            "",
            Posting {
                status: Status::Unmarked,
                account: "assets:cash".into(),
                amount: Some(Amount {
                    currency: "$".into(),
                    value: dec!(100),
                }),
                unit_price: None,
                total_price: None,
            }
        )
    )
}

#[test]
fn test_correct_termination_parse_posting() {
    assert_eq!(
        parse_posting(" assets:cash\n2008/06/01 gift\n  assets:bank:checking  $1").unwrap(),
        (
            "\n2008/06/01 gift\n  assets:bank:checking  $1",
            Posting {
                status: Status::Unmarked,
                account: "assets:cash".into(),
                amount: None,
                unit_price: None,
                total_price: None,
            }
        )
    )
}

#[test]
fn test_parse_posting_with_status() {
    assert_eq!(
        parse_posting(" ! assets:cash  $100").unwrap(),
        (
            "",
            Posting {
                status: Status::Pending,
                account: "assets:cash".into(),
                amount: Some(Amount {
                    currency: "$".into(),
                    value: dec!(100)
                }),
                unit_price: None,
                total_price: None,
            }
        )
    )
}

#[test]
fn test_parse_posting_without_amount() {
    assert_eq!(
        parse_posting(" assets:cash").unwrap(),
        (
            "",
            Posting {
                status: Status::Unmarked,
                account: "assets:cash".into(),
                amount: None,
                unit_price: None,
                total_price: None,
            }
        )
    )
}

#[test]
fn test_parse_posting_no_starting_space() {
    assert_eq!(
        parse_posting("assets:cash").unwrap_err().to_string(),
        nom::Err::Error(HLParserError::Parse(
            "assets:cash".to_string(),
            ErrorKind::Space
        ))
        .to_string()
    )
}

#[test]
fn test_parse_posting_with_unit_price() {
    assert_eq!(
        parse_posting(" ! assets:cash  $100 @ EUR0.94").unwrap(),
        (
            "",
            Posting {
                status: Status::Pending,
                account: "assets:cash".into(),
                amount: Some(Amount {
                    currency: "$".into(),
                    value: dec!(100)
                }),
                unit_price: Some(Amount {
                    currency: "EUR".into(),
                    value: dec!(0.94),
                }),
                total_price: None,
            }
        )
    )
}

#[test]
fn test_parse_posting_with_total_price() {
    assert_eq!(
        parse_posting(" ! assets:cash  $100 @@ €93,89").unwrap(),
        (
            "",
            Posting {
                status: Status::Pending,
                account: "assets:cash".into(),
                amount: Some(Amount {
                    currency: "$".into(),
                    value: dec!(100)
                }),
                unit_price: None,
                total_price: Some(Amount {
                    currency: "€".into(),
                    value: dec!(93.89),
                }),
            }
        )
    )
}
