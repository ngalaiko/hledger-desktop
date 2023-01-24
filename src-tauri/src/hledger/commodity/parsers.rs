use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending, space0, space1},
    combinator::{consumed, eof, map, opt},
    sequence::{delimited, separated_pair, terminated},
};

use crate::hledger::{
    amount::parsers::{parse_currency_string, parse_money_amount},
    HLParserIResult,
};

use super::types::Commodity;

fn parse_commodity_directive_single_line(input: &str) -> HLParserIResult<&str, Commodity> {
    let (tail, _) = terminated(tag("commodity"), space1)(input)?;
    alt((
        map(
            consumed(separated_pair(
                consumed(parse_money_amount),
                space0,
                parse_currency_string,
            )),
            |(full_format, (_, name))| Commodity {
                name: name.to_string(),
                format: Some(full_format.to_string()),
            },
        ),
        map(
            consumed(separated_pair(
                parse_currency_string,
                space0,
                opt(consumed(parse_money_amount)),
            )),
            |(full_format, (name, format))| Commodity {
                name: name.to_string(),
                format: format.map(|_| full_format.to_string()),
            },
        ),
    ))(tail)
}

fn parse_commodity_directive_multi_line(input: &str) -> HLParserIResult<&str, Commodity> {
    let (tail, _) = terminated(tag("commodity"), space1)(input)?;
    let (tail, name) = terminated(parse_currency_string, line_ending)(tail)?;
    let (tail, _) = delimited(space1, tag("format"), space1)(tail)?;
    let (tail, full_format) = not_line_ending(tail)?;

    Ok((
        tail,
        Commodity {
            name: name.to_string(),
            format: Some(full_format.to_string()),
        },
    ))
}

// TODO: add commodity directive comments
pub fn parse_commodity_directive(input: &str) -> HLParserIResult<&str, Commodity> {
    terminated(
        alt((
            parse_commodity_directive_multi_line,
            parse_commodity_directive_single_line,
        )),
        alt((line_ending, eof)),
    )(input)
}
