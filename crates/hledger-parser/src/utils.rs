use chumsky::prelude::*;

use crate::component::comment::{inline, Comment};
use crate::component::whitespace::whitespace;
use crate::state::State;

pub fn end_of_line<'a>(
) -> impl Parser<'a, &'a str, Option<Comment>, extra::Full<Rich<'a, char>, State, ()>> {
    end_of_line_prefixed(0)
}

pub fn end_of_line_prefixed<'a>(
    prefix_whitespace: usize,
) -> impl Parser<'a, &'a str, Option<Comment>, extra::Full<Rich<'a, char>, State, ()>> {
    whitespace()
        .repeated()
        .at_least(prefix_whitespace)
        .ignore_then(inline().map(Some))
        .or(whitespace().repeated().map(|()| None))
}
