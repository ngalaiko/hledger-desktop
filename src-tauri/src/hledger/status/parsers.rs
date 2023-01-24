use nom::{branch::alt, character::complete::char, combinator::opt, sequence::terminated};

use crate::hledger::HLParserIResult;

use super::types::Status;

pub fn parse_status(input: &str) -> HLParserIResult<&str, Status> {
    let (tail, status) = opt(terminated(alt((char('!'), char('*'))), char(' ')))(input)?;

    let status = match status {
        Some('!') => Status::Pending,
        Some('*') => Status::Cleared,
        _ => Status::Unmarked,
    };

    Ok((tail, status))
}
