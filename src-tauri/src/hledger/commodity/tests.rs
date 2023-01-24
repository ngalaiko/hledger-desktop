use crate::hledger::commodity::types::Commodity;

use super::parsers::parse_commodity_directive;

#[test]
fn test_parse_commodity_directive_single_line_name_prefix() {
    assert_eq!(
        parse_commodity_directive("commodity $1000.00").unwrap(),
        (
            "",
            Commodity {
                name: "$".to_string(),
                format: Some("$1000.00".to_string()),
            }
        )
    );
    assert_eq!(
        parse_commodity_directive("commodity $ 1000.00").unwrap(),
        (
            "",
            Commodity {
                name: "$".to_string(),
                format: Some("$ 1000.00".to_string()),
            }
        )
    );
}

#[test]
fn test_parse_commodity_directive_single_line_name_suffix() {
    assert_eq!(
        parse_commodity_directive("commodity 1000.00USD").unwrap(),
        (
            "",
            Commodity {
                name: "USD".to_string(),
                format: Some("1000.00USD".to_string()),
            }
        )
    );
    assert_eq!(
        parse_commodity_directive("commodity 1000.00 USD").unwrap(),
        (
            "",
            Commodity {
                name: "USD".to_string(),
                format: Some("1000.00 USD".to_string()),
            }
        )
    )
}

#[test]
fn test_parse_commodity_directive_multi_line() {
    assert_eq!(
        parse_commodity_directive(
            r#"commodity USD
            format 1000.00USD
"#
        )
        .unwrap(),
        (
            "",
            Commodity {
                name: "USD".to_string(),
                format: Some("1000.00USD".to_string()),
            }
        )
    );
}

#[test]
fn test_parse_commodity_name_only() {
    assert_eq!(
        parse_commodity_directive("commodity INR").unwrap(),
        (
            "",
            Commodity {
                name: "INR".to_string(),
                format: None
            }
        )
    )
}
