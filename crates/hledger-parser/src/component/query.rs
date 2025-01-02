mod condition;

use chumsky::prelude::*;

use crate::component::whitespace::whitespace;
use crate::state::State;

use self::condition::condition;
pub use self::condition::{Amount, Condition, Sign};

#[derive(Clone, Debug, PartialEq)]
pub struct Term {
    pub condition: Condition,
    pub is_not: bool,
}

pub fn query<'a>() -> impl Parser<'a, &'a str, Vec<Term>, extra::Full<Rich<'a, char>, State, ()>> {
    term()
        .separated_by(whitespace().repeated().at_least(1))
        .at_least(1)
        .collect::<Vec<_>>()
}

fn term<'a>() -> impl Parser<'a, &'a str, Term, extra::Full<Rich<'a, char>, State, ()>> {
    just("not:")
        .or_not()
        .then(condition().boxed())
        .map(|(is_not, condition)| Term {
            condition,
            is_not: is_not.is_some(),
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
            Ok(vec![Term {
                is_not: false,
                condition: Condition::Account(String::from("personal care")),
            }])
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
            Ok(vec![Term {
                is_not: false,
                condition: Condition::Account(String::from("expenses:dining")),
            }])
        );
    }

    #[test]
    fn single_simple_term() {
        let result = query().then_ignore(end()).parse("dining").into_result();
        assert_eq!(
            result,
            Ok(vec![Term {
                is_not: false,
                condition: Condition::Account(String::from("dining")),
            }])
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
            Ok(vec![
                Term {
                    is_not: false,
                    condition: Condition::Account(String::from("dining")),
                },
                Term {
                    is_not: false,
                    condition: Condition::Account(String::from("groceries")),
                }
            ])
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
            Ok(vec![Term {
                is_not: true,
                condition: Condition::Account(String::from("opening closing")),
            }])
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
            Ok(vec![Term {
                is_not: false,
                condition: Condition::Description(String::from("opening|closing")),
            }])
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
            Ok(vec![Term {
                is_not: true,
                condition: Condition::Description(String::from("opening|closing")),
            }])
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
            Ok(vec![
                Term {
                    is_not: false,
                    condition: Condition::Account(String::from("account")),
                },
                Term {
                    is_not: false,
                    condition: Condition::Account(String::from("testing account")),
                },
                Term {
                    is_not: false,
                    condition: Condition::Currency(String::from("\\$")),
                },
                Term {
                    is_not: true,
                    condition: Condition::Description(String::from("opening|closing")),
                }
            ])
        );
    }
}
