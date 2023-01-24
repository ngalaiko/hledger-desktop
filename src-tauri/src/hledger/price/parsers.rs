use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, space1},
    combinator::eof,
    sequence::terminated,
};

use crate::hledger::{
    amount::parsers::{parse_amount, parse_currency_string},
    date::parsers::parse_date,
    HLParserIResult,
};

use super::types::Price;

pub fn parse_price(input: &str) -> HLParserIResult<&str, Price> {
    let (tail, _) = terminated(tag("P"), space1)(input)?;
    let (tail, (date, _)) = terminated(parse_date, space1)(tail)?;
    let (tail, commodity) = terminated(parse_currency_string, space1)(tail)?;
    let (tail, amount) = parse_amount(tail)?;
    let (tail, _) = alt((line_ending, eof))(tail)?;

    Ok((
        tail,
        Price {
            commodity: commodity.into(),
            date,
            amount,
        },
    ))
}
