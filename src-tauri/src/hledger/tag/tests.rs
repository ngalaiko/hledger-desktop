use crate::HLParserError;

use super::{parsers::parse_tag, types::Tag};

#[test]
fn test_parse_tag_with_space() {
    let err = parse_tag("not a tag:").unwrap_err().to_string();
    let expected_err = nom::Err::Error(HLParserError::Parse(
        " a tag:".to_string(),
        nom::error::ErrorKind::Tag,
    ))
    .to_string();
    assert_eq!(err, expected_err,)
}

#[test]
fn test_parse_tag_no_value() {
    assert_eq!(
        parse_tag("cash:").unwrap(),
        (
            "",
            Tag {
                name: "cash".into(),
                value: None,
            }
        )
    )
}

#[test]
fn test_parse_tag_with_value() {
    assert_eq!(
        parse_tag("cash:atm").unwrap(),
        (
            "",
            Tag {
                name: "cash".into(),
                value: Some("atm".into()),
            }
        )
    )
}

#[test]
fn test_parse_tag_unicode_no_value() {
    assert_eq!(
        parse_tag("кеш:").unwrap(),
        (
            "",
            Tag {
                name: "кеш".into(),
                value: None,
            }
        )
    )
}
