use nom::{
    bytes::complete::take_until1,
    character::complete::{char, space0},
    sequence::{delimited, pair},
};

use crate::hledger::HLParserIResult;

pub fn parse_code(input: &str) -> HLParserIResult<&str, &str> {
    delimited(
        pair(char('('), space0),
        take_until1(")"),
        pair(space0, char(')')),
    )(input)
}
