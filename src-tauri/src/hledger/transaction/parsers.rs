use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending, space0},
    combinator::{map, opt, value},
    multi::{many0, separated_list0},
    sequence::{preceded, terminated},
};

use crate::hledger::{
    code::parsers::parse_code,
    comment::parsers::{parse_line_comment, parse_transaction_comment},
    date::parsers::parse_date,
    description::parsers::parse_description,
    journal::types::Value,
    posting::parsers::parse_posting,
    status::parsers::parse_status,
    tag::{parsers::parse_tag, types::Tag},
    utils::split_on_space_before_char,
    HLParserIResult, Posting,
};

use super::types::Transaction;

fn parse_comments_tags(input: &str) -> HLParserIResult<&str, (Option<&str>, Vec<Tag>)> {
    let (comment_input, tags_input) = split_on_space_before_char(input, ':');
    let comment = match comment_input.trim().len() {
        0 => None,
        _ => Some(comment_input.trim()),
    };
    let (tail, tags) = separated_list0(terminated(tag(","), space0), parse_tag)(tags_input)
        .map_err(nom::Err::convert)?;

    Ok((tail, (comment, tags)))
}

pub fn parse_transaction(input: &str) -> HLParserIResult<&str, Transaction> {
    let (tail, (primary_date, secondary_date)) =
        terminated(parse_date, space0)(input).map_err(nom::Err::convert)?;
    let (tail, status) = parse_status(tail).map_err(nom::Err::convert)?;
    let (tail, code) = opt(parse_code)(tail).map_err(nom::Err::convert)?;
    let (tail, rest_of_line) = terminated(opt(not_line_ending), line_ending)(tail)?;

    // hmmmm...
    let rest_of_line = rest_of_line.unwrap_or("");
    let (description_input, comment_and_tags_input) =
        rest_of_line.split_at(rest_of_line.find(';').unwrap_or(rest_of_line.len()));
    let (_, description) = parse_description(description_input).map_err(nom::Err::convert)?;
    let (_, comment_and_tags) =
        opt(parse_transaction_comment)(comment_and_tags_input).map_err(nom::Err::convert)?;

    let (_, (_comment, tags)) =
        parse_comments_tags(comment_and_tags.unwrap_or("")).map_err(nom::Err::convert)?;
    let (tail, postings) = many0(alt((
        terminated(
            value(Value::Ignore, preceded(space0, parse_line_comment)),
            line_ending,
        ),
        terminated(map(parse_posting, Value::Posting), line_ending),
    )))(tail)
    .map_err(nom::Err::convert)?;

    let transaction = Transaction {
        primary_date,
        secondary_date,
        code: code.map(str::to_string),
        status,
        description,
        tags,
        postings: postings
            .iter()
            .cloned()
            .filter_map(|line| line.try_into().ok())
            .collect::<Vec<Posting>>(),
    };

    transaction.validate().map_err(nom::Err::Error)?;

    Ok((tail, transaction))
}
