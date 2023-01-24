use std::{path::PathBuf, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, not_line_ending, space0, space1},
    combinator::{all_consuming, eof, map, map_res, value},
    multi::many_till,
    sequence::{preceded, terminated, tuple},
    Finish,
};

use crate::hledger::{
    account::{parsers::parse_account_directive, types::Account},
    comment::parsers::parse_line_comment,
    commodity::{parsers::parse_commodity_directive, types::Commodity},
    include::{parsers::parse_include_directive, types::Include},
    price::{parsers::parse_price, types::Price},
    transaction::{parsers::parse_transaction, types::Transaction},
    HLParserError, HLParserIResult,
};

use super::types::{Journal, Value};

pub fn parse_include_statement(input: &str) -> HLParserIResult<&str, PathBuf> {
    let (rest, path) =
        preceded(tuple((tag("include"), space1)), alt((not_line_ending, eof)))(input)?;
    let path = PathBuf::from_str(path)
        .map_err(|_| nom::Err::Error(HLParserError::IncludePath(path.to_owned())))?;
    Ok((rest, path))
}

pub fn parse_comment_value(input: &str) -> HLParserIResult<&str, Value> {
    value(
        Value::Ignore,
        terminated(parse_line_comment, alt((line_ending, eof))),
    )(input)
}

pub fn parse_empty_line(input: &str) -> HLParserIResult<&str, Value> {
    value(Value::Ignore, terminated(space0, alt((line_ending, eof))))(input)
}

pub fn parse_journal_contents(
    input: &str,
    base_path: PathBuf,
) -> HLParserIResult<&str, Vec<Value>> {
    all_consuming(map(
        many_till(
            alt((
                map(parse_transaction, Value::Transaction),
                parse_comment_value,
                parse_empty_line,
                map(parse_price, Value::Price),
                map(parse_account_directive, |v| Value::Account(v.into())),
                map(parse_commodity_directive, Value::Commodity),
                map(parse_include_directive, |v| Value::Include(v.into())),
                map_res::<_, _, _, _, nom::Err<HLParserError>, _, _>(
                    parse_include_statement,
                    |v| {
                        let path: PathBuf = base_path.clone().join(v);
                        if !path.exists() {
                            return Err(nom::Err::Error(HLParserError::IncludePath(
                                path.to_str().unwrap_or("").to_owned(),
                            )));
                        }
                        let included_journal_contents = std::fs::read_to_string(path.clone())
                            .map_err(|e| nom::Err::Error(HLParserError::IO(e)))?;
                        let (_, values) = parse_journal_contents(
                            &included_journal_contents,
                            path.parent()
                                .map(|v| v.to_owned())
                                .expect("Cannot get parent directory"),
                        )?;
                        Ok(Value::Included(values))
                    },
                ),
            )),
            eof,
        ),
        |(v, _)| v,
    ))(input)
}

pub fn flatten_values(values: Vec<Value>) -> Vec<Value> {
    values
        .into_iter()
        .flat_map(|v| match v {
            Value::Included(contents) => flatten_values(contents),
            _ => vec![v],
        })
        .collect()
}

pub fn parse_journal(input: &str, base_path: Option<PathBuf>) -> Result<Journal, HLParserError> {
    let (_, values) =
        parse_journal_contents(input, base_path.unwrap_or(std::env::current_dir()?)).finish()?;
    let values = flatten_values(values);

    Ok(Journal::new(
        values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Transaction>>(),
        values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Account>>(),
        values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Price>>(),
        values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Commodity>>(),
        values
            .iter()
            .cloned()
            .filter_map(|v| v.try_into().ok())
            .collect::<Vec<Include>>(),
    ))
}
