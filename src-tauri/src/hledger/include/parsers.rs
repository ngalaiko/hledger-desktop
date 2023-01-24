use std::path::PathBuf;

use nom::{
    bytes::complete::tag,
    character::complete::{not_line_ending, space1},
    sequence::{preceded, tuple},
};

use crate::hledger::HLParserIResult;

pub fn parse_include_directive(input: &str) -> HLParserIResult<&str, PathBuf> {
    let (tail, path) = preceded(tuple((tag("include"), space1)), not_line_ending)(input)?;
    return Ok((tail, PathBuf::from(path)));
}
