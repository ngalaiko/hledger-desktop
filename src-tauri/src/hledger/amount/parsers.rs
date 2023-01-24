use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::take_till,
    character::complete::{char, digit1, space0},
    combinator::opt,
    error::ErrorKind,
    multi::separated_list1,
    sequence::{terminated, tuple},
};
use rust_decimal::Decimal;

use crate::hledger::{
    utils::{in_quotes, is_char_digit, is_char_minus, is_char_newline, is_char_space},
    HLParserError, HLParserIResult,
};

use super::types::{Amount, AmountSign};

pub fn parse_money_amount(input: &str) -> HLParserIResult<&str, Decimal> {
    let (tail, (mut parts, fin)) = tuple((
        separated_list1(alt((char(','), char('.'), char(' '))), digit1),
        opt(alt((char('.'), char(',')))),
    ))(input)?;

    let scale = if parts.len() > 1 && fin.is_none() {
        parts.pop()
    } else {
        Some("0")
    };
    let num = parts
        .iter()
        .map(|&id| id.to_string())
        .collect::<String>()
        .parse::<i64>()
        .unwrap_or(0);

    let value = format!("{}.{}", num, scale.unwrap_or("0"));
    let value = Decimal::from_str(&value)
        .map_err(|_| nom::Err::Error(HLParserError::Parse(value.to_string(), ErrorKind::Float)))?;

    Ok((tail, value))
}

fn parse_sign(input: &str) -> HLParserIResult<&str, AmountSign> {
    let (tail, char) = opt(char('-'))(input)?;
    let sign = match char {
        Some(_) => AmountSign::Minus,
        None => AmountSign::Plus,
    };
    Ok((tail, sign))
}

pub fn parse_currency_string(input: &str) -> HLParserIResult<&str, &str> {
    alt((
        in_quotes,
        take_till(|c| {
            is_char_digit(c) || is_char_minus(c) || is_char_space(c) || is_char_newline(c)
        }),
    ))(input)
    .map_err(nom::Err::convert)
}

fn parse_amount_prefix_currency(input: &str) -> HLParserIResult<&str, Amount> {
    let (tail, sign) = terminated(parse_sign, space0)(input)?;
    let (tail, currency) = parse_currency_string(tail)?;
    let (tail, sign) = match sign {
        AmountSign::Minus => (tail, sign),
        AmountSign::Plus => terminated(parse_sign, space0)(tail)?,
    };

    let (tail, mut value) = parse_money_amount(tail)?;
    if sign == AmountSign::Minus {
        value.set_sign_negative(true);
    }

    Ok((
        tail,
        Amount {
            currency: currency.trim().into(),
            value,
        },
    ))
}

fn parse_amount_suffix_currency(input: &str) -> HLParserIResult<&str, Amount> {
    let (tail, sign) = terminated(parse_sign, space0)(input)?;
    let (tail, mut value) = terminated(parse_money_amount, space0)(tail)?;
    if sign == AmountSign::Minus {
        value.set_sign_negative(true);
    }

    let (tail, currency) = parse_currency_string(tail)?;

    Ok((
        tail,
        Amount {
            currency: currency.trim().into(),
            value,
        },
    ))
}

pub fn parse_amount(input: &str) -> HLParserIResult<&str, Amount> {
    alt((
        parse_amount_suffix_currency, // this needs to go first
        parse_amount_prefix_currency,
    ))(input)
}
