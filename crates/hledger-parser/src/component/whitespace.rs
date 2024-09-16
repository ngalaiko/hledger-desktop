use chumsky::prelude::*;

use crate::state::State;

pub fn whitespace<'a>() -> impl Parser<'a, &'a str, (), extra::Full<Rich<'a, char>, State, ()>> {
    one_of(" \t\u{a0}").ignored()
}
