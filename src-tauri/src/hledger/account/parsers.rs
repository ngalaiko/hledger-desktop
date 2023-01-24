use nom::{
    bytes::complete::tag,
    character::complete::{not_line_ending, space1},
    combinator::verify,
    sequence::{preceded, tuple},
};

use crate::hledger::HLParserIResult;

pub fn parse_account_directive(input: &str) -> HLParserIResult<&str, &str> {
    verify(
        preceded(tuple((tag("account"), space1)), not_line_ending),
        |account_name: &str| !account_name.contains("  "),
    )(input)
}
