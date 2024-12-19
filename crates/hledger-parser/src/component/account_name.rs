use std::fmt;

use chumsky::prelude::*;

use crate::state::State;

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct AccountName(Vec<String>);

impl AccountName {
    pub fn from_parts(parts: &[String]) -> Self {
        Self(parts.to_vec())
    }
}

impl std::fmt::Display for AccountName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(":"))
    }
}

pub fn account_name<'a>(
) -> impl Parser<'a, &'a str, AccountName, extra::Full<Rich<'a, char>, State, ()>> {
    let regular_char = any()
        .and_is(text::newline().not())
        .and_is(just(":").not()) // forbidden, because it separates account parts
        .and_is(just("  ").not()) // forbidden, because it separates inline account comment
        .and_is(just(")").not()) // forbidden, because it indicates virtual posting
        .map(|c| format!("{c}"));

    // do not allow closing parenthesis in the end of account name, but allow them in the middle
    let paren_with_following = just(')').then(regular_char).map(|(p, c)| format!("{p}{c}"));
    let valid_segment = paren_with_following.or(regular_char);

    let part = valid_segment.repeated().at_least(1).collect::<Vec<_>>();
    part.separated_by(just(":"))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|parts| {
            parts
                .iter()
                .map(|s| s.join("").trim().to_string())
                .collect::<Vec<String>>()
        })
        .map(|parts| AccountName::from_parts(&parts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_simple() {
        let result = account_name()
            .then_ignore(end())
            .parse("account")
            .into_result();
        assert_eq!(
            result,
            Ok(AccountName::from_parts(&[String::from("account")]))
        );
    }

    #[test]
    fn ok_complex() {
        let result = account_name()
            .then_ignore(end())
            .parse("assets:with (brac\"kets) in:name")
            .into_result();
        assert_eq!(
            result,
            Ok(AccountName::from_parts(&[
                String::from("assets"),
                String::from("with (brac\"kets) in"),
                String::from("name"),
            ]))
        );
    }
}
