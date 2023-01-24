use nom::{
    bytes::complete::{tag, take_while},
    sequence::terminated,
};

use crate::hledger::{utils::is_char_alphanumeric, HLParserIResult};

use super::types::Tag;

pub fn parse_tag(input: &str) -> HLParserIResult<&str, Tag> {
    let (tail, name) = terminated(
        take_while(|c| is_char_alphanumeric(c) || c == '-'),
        tag(":"),
    )(input)?;
    let (tail, value) = take_while(|c| c != ',')(tail)?;
    let value = value.trim();
    let value = match value.len() {
        0 => None,
        _ => Some(value),
    };

    Ok((
        tail,
        Tag {
            name: name.into(),
            value: value.map(str::trim).map(str::to_string),
        },
    ))
}
