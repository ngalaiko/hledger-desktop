use crate::{
    hledger::comment::parsers::{parse_line_comment, parse_transaction_comment},
    HLParserError,
};

#[test]
fn test_parse_line_comment() {
    assert_eq!(parse_line_comment("; comment").unwrap(), ("", "comment"));
    assert_eq!(parse_line_comment("* comment").unwrap(), ("", "comment"));
    assert_eq!(parse_line_comment("# comment").unwrap(), ("", "comment"));
}

#[test]
fn test_parse_transaction_comment() {
    assert_eq!(
        parse_transaction_comment("; comment").unwrap(),
        ("", "comment")
    );
    assert_eq!(
        parse_transaction_comment("# comment")
            .unwrap_err()
            .to_string(),
        nom::Err::Error(HLParserError::Parse(
            "# comment".to_owned(),
            nom::error::ErrorKind::Char
        ))
        .to_string()
    );
}

#[test]
fn test_parse_line_comment_empty() {
    assert_eq!(parse_line_comment(";").unwrap(), ("", ""));
    assert_eq!(parse_line_comment(";\n").unwrap(), ("\n", ""));
}
