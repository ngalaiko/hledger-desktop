use crate::HLParserError;

use super::parsers::parse_code;

#[test]
fn test_parse_code() {
    assert_eq!(
        parse_code("(code123-1!) rest").unwrap(),
        (" rest", "code123-1!")
    )
}

#[test]
fn test_parse_invalid_code() {
    assert_eq!(
        parse_code("()").unwrap_err().to_string(),
        nom::Err::Error(HLParserError::Parse(
            ")".to_owned(),
            nom::error::ErrorKind::TakeUntil
        ))
        .to_string()
    )
}
