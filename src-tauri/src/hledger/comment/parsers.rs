use nom::{
    branch::alt,
    character::complete::char,
    character::complete::{not_line_ending, space0},
    combinator::{eof, rest},
    sequence::preceded,
};

use crate::hledger::HLParserIResult;

pub fn parse_line_comment(input: &str) -> HLParserIResult<&str, &str> {
    preceded(
        alt((char('#'), char(';'), char('*'))),
        preceded(space0, alt((not_line_ending, eof))),
    )(input)
}

pub fn parse_transaction_comment(input: &str) -> HLParserIResult<&str, &str> {
    preceded(char(';'), preceded(space0, rest))(input)
}
