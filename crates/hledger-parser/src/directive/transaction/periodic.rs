use chumsky::prelude::*;

use crate::component::period::{period, Period};
use crate::component::status::Status;
use crate::component::whitespace::whitespace;
use crate::directive::transaction::header::header;
use crate::directive::transaction::posting::{posting, Posting};
use crate::state::State;

#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    pub period: Period,
    pub status: Option<Status>,
    pub code: Option<String>,
    pub payee: String,
    pub description: Option<String>,
    pub postings: Vec<Posting>,
}

pub fn transaction<'a>(
) -> impl Parser<'a, &'a str, Transaction, extra::Full<Rich<'a, char>, State, ()>> {
    let header = just("~")
        .ignore_then(whitespace().repeated())
        .ignore_then(period())
        .then_ignore(whitespace().repeated().at_least(2))
        .then(header().or_not());

    header
        .then(
            posting()
                .separated_by(text::newline())
                .allow_leading()
                .collect::<Vec<_>>(),
        )
        .map(|((period, header), postings)| Transaction {
            period,
            status: header.as_ref().and_then(|h| h.status.clone()),
            code: header.as_ref().and_then(|h| h.code.clone()),
            payee: header.as_ref().map_or(String::new(), |h| h.payee.clone()),
            description: header.as_ref().and_then(|h| h.description.clone()),
            postings,
        })
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::component::{account_name::AccountName, amount::Amount, period::interval::Interval};

    use super::*;

    #[test]
    fn simple() {
        let result = transaction()
            .then_ignore(end())
            .parse(
                "~ monthly from 2023-04-15 to 2023-06-16  electricity
    expenses:utilities          $400
    assets:bank:checking",
            )
            .into_result();
        assert_eq!(
            result,
            Ok(Transaction {
                period: Period {
                    interval: Some(Interval::NthMonth(1)),
                    begin: chrono::NaiveDate::from_ymd_opt(2023, 4, 15),
                    end: chrono::NaiveDate::from_ymd_opt(2023, 6, 16),
                },
                code: None,
                status: None,
                payee: String::from("electricity"),
                description: None,
                postings: vec![
                    Posting {
                        status: None,
                        account_name: AccountName::from_parts(&[
                            String::from("expenses"),
                            String::from("utilities")
                        ]),
                        amount: Some(Amount {
                            quantity: Decimal::new(400, 0),
                            commodity: String::from("$"),
                        }),
                        price: None,
                        assertion: None,
                        is_virtual: false,
                    },
                    Posting {
                        status: None,
                        account_name: AccountName::from_parts(&[
                            String::from("assets"),
                            String::from("bank"),
                            String::from("checking")
                        ]),
                        amount: None,
                        price: None,
                        assertion: None,
                        is_virtual: false,
                    }
                ],
            })
        );
    }

    #[test]
    fn cheatsheet() {
        let result = transaction()
            .then_ignore(end())
            .parse(
                "~ monthly  set budget goals  ; <- Note, 2+ spaces before the description.
    (expenses:rent)      $1000
    (expenses:food)       $500",
            )
            .into_result();
        assert_eq!(
            result,
            Ok(Transaction {
                period: Period {
                    interval: Some(Interval::NthMonth(1)),
                    begin: None,
                    end: None,
                },
                code: None,
                status: None,
                payee: String::from("set budget goals"),
                description: None,
                postings: vec![
                    Posting {
                        status: None,
                        account_name: AccountName::from_parts(&[
                            String::from("expenses"),
                            String::from("rent")
                        ]),
                        amount: Some(Amount {
                            quantity: Decimal::new(1000, 0),
                            commodity: String::from("$"),
                        }),
                        price: None,
                        assertion: None,
                        is_virtual: true,
                    },
                    Posting {
                        status: None,
                        account_name: AccountName::from_parts(&[
                            String::from("expenses"),
                            String::from("food")
                        ]),
                        amount: Some(Amount {
                            quantity: Decimal::new(500, 0),
                            commodity: String::from("$"),
                        }),
                        price: None,
                        assertion: None,
                        is_virtual: true,
                    }
                ],
            })
        );
    }
}
