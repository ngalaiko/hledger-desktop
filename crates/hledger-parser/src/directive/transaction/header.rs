use chumsky::prelude::*;

use crate::component::whitespace::whitespace;
use crate::directive::transaction::status::{status, Status};
use crate::state::State;
use crate::utils::end_of_line;

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub status: Option<Status>,
    pub code: Option<String>,
    pub payee: String,
    pub description: Option<String>,
}

pub fn header<'a>() -> impl Parser<'a, &'a str, Header, extra::Full<Rich<'a, char>, State, ()>> {
    let code = any()
        .and_is(text::newline().not())
        .and_is(just(")").not()) // forbidden, because it indicates end of the code
        .repeated()
        .at_least(1)
        .collect::<String>()
        .delimited_by(just('('), just(')'));

    let payee = any()
        .and_is(text::newline().not())
        .and_is(just("|").not()) // forbidden, because it is a description separator
        .and_is(just(";").not()) // forbidden, because it indicates comment
        .repeated()
        .collect::<String>();

    let description = just("|").ignore_then(whitespace().repeated()).ignore_then(
        any()
            .and_is(text::newline().not())
            .and_is(just(";").not()) // forbidden, because it indicates comment
            .repeated()
            .collect::<String>(),
    );

    status()
        .or_not()
        .then(whitespace().repeated().ignore_then(code).or_not())
        .then(whitespace().repeated().ignore_then(payee))
        .then(whitespace().repeated().ignore_then(description).or_not())
        .then_ignore(end_of_line())
        .map(|(((status, code), payee), description)| Header {
            status,
            code,
            payee: payee.trim().to_string(),
            description,
        })
}
