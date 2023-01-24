use rust_decimal_macros::dec;

use crate::hledger::amount::{
    parsers::{parse_amount, parse_currency_string},
    types::Amount,
};

use super::parsers::parse_money_amount;

#[test]
fn test_parse_amount_currency_prefixed() {
    assert_eq!(
        parse_amount("$100").unwrap(),
        (
            "",
            Amount {
                currency: "$".into(),
                value: dec!(100)
            }
        )
    );
    assert_eq!(
        parse_amount("$ 100").unwrap(),
        (
            "",
            Amount {
                currency: "$".into(),
                value: dec!(100)
            }
        )
    );
    assert_eq!(
        parse_amount("$  100").unwrap(),
        (
            "",
            Amount {
                currency: "$".into(),
                value: dec!(100)
            }
        )
    );
    assert_eq!(
        parse_amount("\"silver coins\" 100").unwrap(),
        (
            "",
            Amount {
                currency: "silver coins".into(),
                value: dec!(100)
            }
        )
    );
    assert_eq!(
        parse_amount("\"silver coins\"  100").unwrap(),
        (
            "",
            Amount {
                currency: "silver coins".into(),
                value: dec!(100)
            }
        )
    );
}

#[test]
fn test_parse_amount_currency_suffixed() {
    assert_eq!(
        parse_amount("100EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(100)
            }
        )
    );
    assert_eq!(
        parse_amount("100 EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(100)
            }
        )
    );
    assert_eq!(
        parse_amount("100  EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(100)
            }
        )
    );
    assert_eq!(
        parse_amount("100 \"silver coins\"").unwrap(),
        (
            "",
            Amount {
                currency: "silver coins".into(),
                value: dec!(100)
            }
        )
    );
    assert_eq!(
        parse_amount("100  \"silver coins\"").unwrap(),
        (
            "",
            Amount {
                currency: "silver coins".into(),
                value: dec!(100)
            }
        )
    );
}

#[test]
fn test_parse_amount_negative_currency_suffixed() {
    assert_eq!(
        parse_amount("-100EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(-100)
            }
        )
    );
    assert_eq!(
        parse_amount("- 100EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(-100)
            }
        )
    );
    assert_eq!(
        parse_amount("-  100EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(-100)
            }
        )
    );
    assert_eq!(
        parse_amount("-100 EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(-100)
            }
        )
    );
    assert_eq!(
        parse_amount("-100  EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(-100)
            }
        )
    );
    assert_eq!(
        parse_amount("- 100 EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(-100)
            }
        )
    );
    assert_eq!(
        parse_amount("-  100 EUR").unwrap(),
        (
            "",
            Amount {
                currency: "EUR".into(),
                value: dec!(-100)
            }
        )
    );
}

#[test]
fn test_parse_amount_negative_currency_prefixed() {
    assert_eq!(
        parse_amount("-$100").unwrap(),
        (
            "",
            Amount {
                currency: "$".into(),
                value: dec!(-100)
            }
        )
    );
    assert_eq!(
        parse_amount("$-100").unwrap(),
        (
            "",
            Amount {
                currency: "$".into(),
                value: dec!(-100)
            }
        )
    );
}

#[test]
fn test_parse_money_amount_int() {
    assert_eq!(parse_money_amount("100EUR").unwrap(), ("EUR", dec!(100)))
}

#[test]
fn test_parse_money_amount_double() {
    assert_eq!(parse_money_amount("100.00EUR").unwrap(), ("EUR", dec!(100)));
    assert_eq!(parse_money_amount("100,00EUR").unwrap(), ("EUR", dec!(100)));
}

#[test]
fn test_parse_money_amount_negative() {
    assert_eq!(
        parse_money_amount("-100.00").unwrap(),
        ("", dec!(-100.0))
    );
}

#[test]
fn test_parse_money_amount_empty_fraction() {
    assert_eq!(
        parse_money_amount("100,000.").unwrap(),
        ("", dec!(100000.0))
    );
}

#[test]
fn test_parse_money_amount_thousand_separator() {
    assert_eq!(
        parse_money_amount("100,000.00EUR").unwrap(),
        ("EUR", dec!(100000.0))
    );
}

#[test]
fn test_parse_money_amount_small() {
    assert_eq!(
        parse_money_amount("0.007EUR").unwrap(),
        ("EUR", dec!(0.007))
    );
}

#[test]
fn test_parse_fractional_amount() {
    assert_eq!(
        parse_amount("$100.95").unwrap(),
        (
            "",
            Amount {
                currency: "$".into(),
                value: dec!(100.95),
            }
        )
    )
}

#[test]
fn test_parse_currency_string_symbol() {
    assert_eq!(parse_currency_string("$").unwrap(), ("", "$"));
    assert_eq!(parse_currency_string("$ ").unwrap(), (" ", "$"));
}

#[test]
fn test_parse_currency_string_quotes() {
    assert_eq!(
        parse_currency_string("\"Imaginary money\"").unwrap(),
        ("", "Imaginary money")
    );
    assert_eq!(
        parse_currency_string("\"Imaginary money\" ").unwrap(),
        (" ", "Imaginary money")
    );
}

#[test]
fn test_parse_currency_string_iso() {
    assert_eq!(parse_currency_string("USD").unwrap(), ("", "USD"));
    assert_eq!(parse_currency_string("USD ").unwrap(), (" ", "USD"));
}
