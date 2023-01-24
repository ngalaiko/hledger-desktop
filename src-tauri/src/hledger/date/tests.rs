use chrono::NaiveDate;
use nom::error::ErrorKind;

use crate::hledger::{date::parsers::parse_date, HLParserError};

#[test]
fn test_parse_date_dash() {
    assert_eq!(
        parse_date("2020-01-01").unwrap(),
        ("", (NaiveDate::from_ymd(2020, 1, 1), None))
    );
}

#[test]
fn test_parse_date_slash() {
    assert_eq!(
        parse_date("2020/01/01").unwrap(),
        ("", (NaiveDate::from_ymd(2020, 1, 1), None))
    );
}

#[test]
fn test_parse_date_dot() {
    assert_eq!(
        parse_date("2020.01.01").unwrap(),
        ("", (NaiveDate::from_ymd(2020, 1, 1), None))
    );
}

#[test]
fn test_parse_date_invalid_month() {
    assert_eq!(
        parse_date("2020.13.01").unwrap_err().to_string(),
        nom::Err::Error(HLParserError::Parse(
            "2020.13.01".to_string(),
            ErrorKind::Tag
        ))
        .to_string()
    );
    assert_eq!(
        parse_date("2020.00.01").unwrap_err().to_string(),
        nom::Err::Error(HLParserError::Parse(
            "2020.00.01".to_string(),
            ErrorKind::Tag
        ))
        .to_string()
    );
}

#[test]
fn test_parse_date_invalid_day() {
    assert_eq!(
        parse_date("2021.02.29").unwrap_err().to_string(),
        nom::Err::Error(HLParserError::Parse(
            "2021.02.29".to_string(),
            ErrorKind::Tag
        ))
        .to_string()
    );
    assert_eq!(
        parse_date("2021.02.60").unwrap_err().to_string(),
        nom::Err::Error(HLParserError::Parse(
            "2021.02.60".to_string(),
            ErrorKind::Tag
        ))
        .to_string()
    );
}

#[test]
fn test_parse_date_mix_separator() {
    assert_eq!(
        parse_date("2021/02.29").unwrap_err().to_string(),
        nom::Err::Error(HLParserError::Parse(
            "2021/02.29".to_string(),
            ErrorKind::MapRes
        ))
        .to_string()
    );
}

#[test]
fn test_parse_date_secondary_dash() {
    assert_eq!(
        parse_date("2021-01-01=2021-01-05").unwrap(),
        (
            "",
            (
                NaiveDate::from_ymd(2021, 1, 1),
                Some(NaiveDate::from_ymd(2021, 1, 5))
            )
        ),
    )
}

#[test]
fn test_parse_date_secondary_slash() {
    assert_eq!(
        parse_date("2021/01/01=2021/01/05").unwrap(),
        (
            "",
            (
                NaiveDate::from_ymd(2021, 1, 1),
                Some(NaiveDate::from_ymd(2021, 1, 5))
            )
        ),
    )
}

#[test]
fn test_parse_date_secondary_dot() {
    assert_eq!(
        parse_date("2021.01.01=2021.01.05").unwrap(),
        (
            "",
            (
                NaiveDate::from_ymd(2021, 1, 1),
                Some(NaiveDate::from_ymd(2021, 1, 5))
            )
        ),
    )
}

#[test]
fn test_parse_date_secondary_smart_date() {
    assert_eq!(
        parse_date("2021-01-01=01-05").unwrap(),
        (
            "",
            (
                NaiveDate::from_ymd(2021, 1, 1),
                Some(NaiveDate::from_ymd(2021, 1, 5))
            )
        ),
    );
    assert_eq!(
        parse_date("2021-01-01=12-5").unwrap(),
        (
            "",
            (
                NaiveDate::from_ymd(2021, 1, 1),
                Some(NaiveDate::from_ymd(2021, 12, 5))
            )
        ),
    );
}
