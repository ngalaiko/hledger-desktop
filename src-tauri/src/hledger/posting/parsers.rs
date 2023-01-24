use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until},
    character::complete::{space0, space1},
    combinator::{map_res, opt, peek, success, verify},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};

use crate::hledger::{
    amount::parsers::parse_amount, comment::parsers::parse_line_comment,
    status::parsers::parse_status, HLParserError, HLParserIResult,
};

use super::types::{Posting, PostingComplexAmount};

fn parse_posting_with_amount(input: &str) -> HLParserIResult<&str, (&str, PostingComplexAmount)> {
    let (tail, account_name) = verify(
        terminated(take_until("  "), peek(preceded(space1, parse_amount))),
        |s: &str| !s.contains('\n'),
    )(input)?;
    let (tail, complex_amount) = alt((
        // order of parsers is important
        map_res(
            separated_pair(
                parse_amount,
                delimited(space0, tag("@@"), space0),
                parse_amount,
            ),
            |(amount, total_price)| -> Result<PostingComplexAmount, HLParserError> {
                Ok(PostingComplexAmount {
                    amount: Some(amount),
                    unit_price: None,
                    total_price: Some(total_price),
                })
            },
        ),
        map_res(
            separated_pair(
                parse_amount,
                delimited(space0, tag("@"), space0),
                parse_amount,
            ),
            |(amount, unit_price)| -> Result<PostingComplexAmount, HLParserError> {
                Ok(PostingComplexAmount {
                    amount: Some(amount),
                    unit_price: Some(unit_price),
                    total_price: None,
                })
            },
        ),
        map_res(
            opt(parse_amount),
            |amount| -> Result<PostingComplexAmount, HLParserError> {
                Ok(PostingComplexAmount {
                    amount,
                    unit_price: None,
                    total_price: None,
                })
            },
        ),
    ))(tail)?;

    Ok((tail, (account_name, complex_amount)))
}

fn parse_posting_without_amount(
    input: &str,
) -> HLParserIResult<&str, (&str, PostingComplexAmount)> {
    tuple((
        is_not("\n"),
        success(PostingComplexAmount {
            amount: None,
            unit_price: None,
            total_price: None,
        }),
    ))(input)
}

pub fn parse_posting(input: &str) -> HLParserIResult<&str, Posting> {
    let (tail, (status, (account_name, complex_amount), _comment)) = tuple((
        delimited(space1, parse_status, space0),
        alt((parse_posting_with_amount, parse_posting_without_amount)),
        opt(preceded(space0, parse_line_comment)),
    ))(input)?;

    Ok((
        tail,
        Posting {
            status,
            account: account_name.trim().into(),
            amount: complex_amount.amount,
            unit_price: complex_amount.unit_price,
            total_price: complex_amount.total_price,
        },
    ))
}
