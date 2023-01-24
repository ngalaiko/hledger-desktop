use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::space0,
    combinator::{opt, rest},
    sequence::{delimited, separated_pair},
};

use crate::hledger::{HLParserError, HLParserIResult};

use super::types::Description;

fn parse_only_note(input: &str) -> HLParserIResult<&str, &str> {
    rest(input)
}

fn parse_payee_and_note(input: &str) -> HLParserIResult<&str, (Option<&str>, Option<&str>)> {
    separated_pair(
        opt(alt((take_until(" |"), take_until("|")))),
        delimited(space0, tag("|"), space0),
        opt(parse_only_note),
    )(input)
}

// TODO: this is fugly
pub fn parse_description(input: &str) -> HLParserIResult<&str, Description> {
    match parse_payee_and_note(input) {
        Ok((t, (p, n))) => Ok((
            t,
            Description {
                payee: match p.map(str::trim) {
                    None => None,
                    Some(v) if v.is_empty() => None,
                    Some(v) => Some(v.into()),
                },
                note: match n.map(str::trim) {
                    None => None,
                    Some(v) if v.is_empty() => None,
                    Some(v) => Some(v.into()),
                },
            },
        )),
        Err(_) => match opt(parse_only_note)(input) {
            Ok((t, n)) => Ok((
                t,
                Description {
                    payee: None,
                    note: match n.map(str::trim) {
                        Some("") => None,
                        Some(n) => Some(n.into()),
                        None => None,
                    },
                },
            )),
            Err(_) => Err(nom::Err::Error(HLParserError::Parse(
                input.to_owned(),
                nom::error::ErrorKind::Tag,
            ))),
        },
    }
}
