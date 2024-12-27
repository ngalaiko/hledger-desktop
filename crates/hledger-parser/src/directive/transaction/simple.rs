use chumsky::prelude::*;

use crate::component::date::simple::date;
use crate::component::status::Status;
use crate::component::whitespace::whitespace;
use crate::directive::transaction::header::header;
use crate::directive::transaction::posting::{posting, Posting};
use crate::state::State;

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Transaction {
    pub date: chrono::NaiveDate,
    pub status: Option<Status>,
    pub code: Option<String>,
    pub payee: String,
    pub note: Option<String>,
    pub postings: Vec<Posting>,
    pub position: std::ops::Range<usize>,
}

pub fn transaction<'a>(
) -> impl Parser<'a, &'a str, Transaction, extra::Full<Rich<'a, char>, State, ()>> {
    let header = date()
        .then_ignore(whitespace().repeated())
        .then(header().or_not());

    header
        .then(
            posting()
                .separated_by(text::newline())
                .allow_leading()
                .collect::<Vec<_>>(),
        )
        .map_with(|((date, header), postings), e| Transaction {
            date,
            status: header.as_ref().and_then(|h| h.status.clone()),
            code: header.as_ref().and_then(|h| h.code.clone()),
            payee: header.as_ref().map_or(String::new(), |h| h.payee.clone()),
            note: header.as_ref().and_then(|h| h.note.clone()),
            postings,
            position: e.span().into_range(),
        })
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::component::{account_name::AccountName, amount::Amount};

    use super::*;

    #[test]
    fn full() {
        let result = transaction()
            .then_ignore(end())
            .parse(
                "2008/01/01 * (123) salary | january ; transaction comment
                                                 ; same comment second line
    assets:bank:checking   $1  ; posting comment
                               ; same comment second line
    income:salary  ",
            )
            .into_result();
        assert_eq!(
            result,
            Ok(Transaction {
                date: chrono::NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
                code: Some(String::from("123")),
                status: Some(Status::Cleared),
                payee: String::from("salary"),
                note: Some(String::from("january ")),
                postings: vec![
                    Posting {
                        status: None,
                        account_name: AccountName::from_parts(&[
                            String::from("assets"),
                            String::from("bank"),
                            String::from("checking"),
                        ]),
                        amount: Some(Amount {
                            quantity: Decimal::new(1, 0),
                            commodity: String::from("$"),
                        }),
                        price: None,
                        assertion: None,
                        is_virtual: false,
                    },
                    Posting {
                        status: None,
                        account_name: AccountName::from_parts(&[
                            String::from("income"),
                            String::from("salary")
                        ]),
                        amount: None,
                        price: None,
                        assertion: None,
                        is_virtual: false,
                    }
                ],
                position: (0..260),
            })
        );
    }

    #[test]
    fn simple() {
        let result = transaction()
            .then_ignore(end())
            .parse(
                "2008/01/01 salary
    assets:bank:checking   $1
    income:salary  ",
            )
            .into_result();
        assert_eq!(
            result,
            Ok(Transaction {
                date: chrono::NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
                code: None,
                status: None,
                payee: String::from("salary"),
                note: None,
                postings: vec![
                    Posting {
                        status: None,
                        account_name: AccountName::from_parts(&[
                            String::from("assets"),
                            String::from("bank"),
                            String::from("checking"),
                        ]),
                        amount: Some(Amount {
                            quantity: Decimal::new(1, 0),
                            commodity: String::from("$"),
                        }),
                        price: None,
                        assertion: None,
                        is_virtual: false,
                    },
                    Posting {
                        status: None,
                        account_name: AccountName::from_parts(&[
                            String::from("income"),
                            String::from("salary")
                        ]),
                        amount: None,
                        price: None,
                        assertion: None,
                        is_virtual: false,
                    }
                ],
                position: (0..67),
            })
        );
    }

    #[test]
    fn just_date() {
        let result = transaction()
            .then_ignore(end())
            .parse("2008/1/1")
            .into_result();
        assert_eq!(
            result,
            Ok(Transaction {
                date: chrono::NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
                code: None,
                status: None,
                payee: String::new(),
                note: None,
                postings: vec![],
                position: (0..8),
            })
        );
    }
}
