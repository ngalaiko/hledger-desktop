use nom::error::ErrorKind;

use crate::hledger::{account::parsers::parse_account_directive, HLParserError};

#[test]
fn test_parse_account_directive() {
    assert_eq!(
        parse_account_directive("account assets:cash").unwrap(),
        ("", "assets:cash")
    );
}

#[test]
fn test_parse_account_directive_invalid_name() {
    assert_eq!(
        parse_account_directive("account assets:cash  ")
            .unwrap_err()
            .to_string(),
        nom::Err::Error(HLParserError::Parse(
            "account assets:cash  ".to_string(),
            ErrorKind::Verify
        ))
        .to_string()
    );
}
