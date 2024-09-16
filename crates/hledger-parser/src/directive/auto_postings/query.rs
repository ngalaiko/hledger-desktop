use chumsky::prelude::*;

use crate::{component::whitespace::whitespace, state::State};

#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub terms: Vec<Term>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Term {
    pub r#type: Option<String>,
    pub is_not: bool,
    pub value: String,
}

pub fn query<'a>() -> impl Parser<'a, &'a str, Query, extra::Full<Rich<'a, char>, State, ()>> {
    term()
        .separated_by(whitespace().repeated().at_least(1))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|terms| Query { terms })
}

fn term<'a>() -> impl Parser<'a, &'a str, Term, extra::Full<Rich<'a, char>, State, ()>> {
    let value = any()
        .and_is(text::newline().not())
        .and_is(whitespace().not())
        .repeated()
        .at_least(1)
        .collect::<String>();
    let quoted_value = any()
        .and_is(text::newline().not())
        .and_is(just("'").not()) // indicated end of quote
        .repeated()
        .at_least(1)
        .collect::<String>()
        .delimited_by(just("'"), just("'"));
    let r#type = just("date")
        .or(just("status"))
        .or(just("desc"))
        .or(just("cur"))
        .or(just("amt"))
        .then_ignore(just(":"))
        .map(ToString::to_string);

    just("not:")
        .or_not()
        .then(r#type.or_not())
        .then(quoted_value.or(value))
        .map(|((is_not, r#type), value)| Term {
            r#type,
            is_not: is_not.is_some(),
            value,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_quoted_term() {
        let result = query()
            .then_ignore(end())
            .parse("'personal care'")
            .into_result();
        assert_eq!(
            result,
            Ok(Query {
                terms: vec![Term {
                    is_not: false,
                    value: String::from("personal care"),
                    r#type: None,
                }]
            })
        );
    }

    #[test]
    fn single_account_term() {
        let result = query()
            .then_ignore(end())
            .parse("expenses:dining")
            .into_result();
        assert_eq!(
            result,
            Ok(Query {
                terms: vec![Term {
                    is_not: false,
                    value: String::from("expenses:dining"),
                    r#type: None,
                }]
            })
        );
    }

    #[test]
    fn single_simple_term() {
        let result = query().then_ignore(end()).parse("dining").into_result();
        assert_eq!(
            result,
            Ok(Query {
                terms: vec![Term {
                    is_not: false,
                    value: String::from("dining"),
                    r#type: None,
                }]
            })
        );
    }

    #[test]
    fn multiple_simple_terms() {
        let result = query()
            .then_ignore(end())
            .parse("dining groceries")
            .into_result();
        assert_eq!(
            result,
            Ok(Query {
                terms: vec![
                    Term {
                        is_not: false,
                        value: String::from("dining"),
                        r#type: None,
                    },
                    Term {
                        is_not: false,
                        value: String::from("groceries"),
                        r#type: None,
                    }
                ]
            })
        );
    }

    #[test]
    fn not_term() {
        let result = query()
            .then_ignore(end())
            .parse("not:'opening closing'")
            .into_result();
        assert_eq!(
            result,
            Ok(Query {
                terms: vec![Term {
                    is_not: true,
                    value: String::from("opening closing"),
                    r#type: None,
                }]
            })
        );
    }

    #[test]
    fn typed_term() {
        let result = query()
            .then_ignore(end())
            .parse("desc:'opening|closing'")
            .into_result();
        assert_eq!(
            result,
            Ok(Query {
                terms: vec![Term {
                    is_not: false,
                    value: String::from("opening|closing"),
                    r#type: Some(String::from("desc")),
                }]
            })
        );
    }

    #[test]
    fn not_typed_term() {
        let result = query()
            .then_ignore(end())
            .parse("not:desc:'opening|closing'")
            .into_result();
        assert_eq!(
            result,
            Ok(Query {
                terms: vec![Term {
                    is_not: true,
                    value: String::from("opening|closing"),
                    r#type: Some(String::from("desc")),
                }]
            })
        );
    }

    #[test]
    fn complex() {
        let result = query()
            .then_ignore(end())
            .parse("account 'testing account' cur:\\$ not:desc:'opening|closing'")
            .into_result();
        assert_eq!(
            result,
            Ok(Query {
                terms: vec![
                    Term {
                        is_not: false,
                        value: String::from("account"),
                        r#type: None,
                    },
                    Term {
                        is_not: false,
                        value: String::from("testing account"),
                        r#type: None,
                    },
                    Term {
                        is_not: false,
                        value: String::from("\\$"),
                        r#type: Some(String::from("cur")),
                    },
                    Term {
                        is_not: true,
                        value: String::from("opening|closing"),
                        r#type: Some(String::from("desc")),
                    }
                ]
            })
        );
    }
}
