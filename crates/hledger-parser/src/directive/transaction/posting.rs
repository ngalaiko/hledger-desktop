use chumsky::prelude::*;

mod assertion;

use crate::component::account_name::{account_name, AccountName};
use crate::component::amount::{amount, Amount};
use crate::component::price::{amount_price, AmountPrice};
use crate::component::status::{status, Status};
use crate::component::whitespace::whitespace;
use crate::directive::transaction::posting::assertion::assertion;
use crate::state::State;
use crate::utils::end_of_line;

pub use crate::directive::transaction::posting::assertion::Assertion;

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Posting {
    pub status: Option<Status>,
    pub account_name: AccountName,
    pub is_virtual: bool,
    pub amount: Option<Amount>,
    pub price: Option<AmountPrice>,
    pub assertion: Option<Assertion>,
}

#[must_use]
pub fn posting<'a>() -> impl Parser<'a, &'a str, Posting, extra::Full<Rich<'a, char>, State, ()>> {
    let posting_amount = whitespace().repeated().at_least(2).ignore_then(amount());
    let posting_price = whitespace().repeated().ignore_then(amount_price());
    let posting_assertion = whitespace().repeated().ignore_then(assertion());
    let account_name = account_name()
        .delimited_by(just('('), just(')'))
        .map(|name| (name, true))
        .or(account_name().map(|name| (name, false)));
    whitespace()
        .repeated()
        .at_least(1)
        .ignore_then(status().then_ignore(whitespace()).or_not())
        .then(account_name)
        .then(posting_amount.or_not())
        .then(posting_price.or_not())
        .then(posting_assertion.or_not())
        .then_ignore(end_of_line())
        .map(
            |((((status, (account_name, is_virtual)), amount), price), assertion)| Posting {
                status,
                account_name,
                is_virtual,
                amount,
                price,
                assertion,
            },
        )
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn full() {
        let result = posting()
            .then_ignore(end())
            .parse(" ! assets:bank:checking   $1")
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
                status: Some(Status::Pending),
                account_name: AccountName::from_parts(&[
                    String::from("assets"),
                    String::from("bank"),
                    String::from("checking")
                ]),
                amount: Some(Amount {
                    quantity: Decimal::new(1, 0),
                    commodity: String::from("$"),
                }),
                price: None,
                assertion: None,
                is_virtual: false,
            })
        );
    }

    #[test]
    fn no_amount() {
        let result = posting()
            .then_ignore(end())
            .parse(" ! assets:bank:checking")
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
                status: Some(Status::Pending),
                account_name: AccountName::from_parts(&[
                    String::from("assets"),
                    String::from("bank"),
                    String::from("checking")
                ]),
                amount: None,
                price: None,
                assertion: None,
                is_virtual: false,
            })
        );
    }

    #[test]
    fn no_status() {
        let result = posting()
            .then_ignore(end())
            .parse(" assets:bank:checking   $1")
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
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
            })
        );
    }

    #[test]
    fn with_comment() {
        let result = posting()
            .then_ignore(end())
            .parse(
                " assets:bank:checking  ; some comment
                                    ; continuation of the same comment",
            )
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
                status: None,
                account_name: AccountName::from_parts(&[
                    String::from("assets"),
                    String::from("bank"),
                    String::from("checking"),
                ]),
                amount: None,
                price: None,
                assertion: None,
                is_virtual: false,
            })
        );
    }

    #[test]
    fn no_status_no_amount() {
        let result = posting()
            .then_ignore(end())
            .parse(" assets:bank:checking")
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
                status: None,
                account_name: AccountName::from_parts(&[
                    String::from("assets"),
                    String::from("bank"),
                    String::from("checking"),
                ]),
                amount: None,
                price: None,
                assertion: None,
                is_virtual: false,
            })
        );
    }

    #[test]
    fn with_price_assertion() {
        let result = posting()
            .then_ignore(end())
            .parse(" assets:bank:checking  1 EUR@@1 USD=1 USD")
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
                status: None,
                account_name: AccountName::from_parts(&[
                    String::from("assets"),
                    String::from("bank"),
                    String::from("checking"),
                ]),
                amount: Some(Amount {
                    quantity: Decimal::new(1, 0),
                    commodity: String::from("EUR"),
                }),
                price: Some(AmountPrice::Total(Amount {
                    quantity: Decimal::new(1, 0),
                    commodity: String::from("USD"),
                })),
                assertion: Some(Assertion {
                    price: None,
                    amount: Amount {
                        quantity: Decimal::new(1, 0),
                        commodity: String::from("USD"),
                    },
                    is_subaccount_inclusive: false,
                    is_strict: false,
                }),
                is_virtual: false,
            })
        );
    }

    #[test]
    fn with_assertion() {
        let result = posting()
            .then_ignore(end())
            .parse(" assets:bank:checking  1 USD == 1 USD")
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
                status: None,
                account_name: AccountName::from_parts(&[
                    String::from("assets"),
                    String::from("bank"),
                    String::from("checking"),
                ]),
                amount: Some(Amount {
                    quantity: Decimal::new(1, 0),
                    commodity: String::from("USD"),
                }),
                price: None,
                assertion: Some(Assertion {
                    price: None,
                    amount: Amount {
                        quantity: Decimal::new(1, 0),
                        commodity: String::from("USD"),
                    },
                    is_subaccount_inclusive: false,
                    is_strict: true,
                }),
                is_virtual: false,
            })
        );
    }

    #[test]
    fn with_price() {
        let result = posting()
            .then_ignore(end())
            .parse(" assets:bank:checking  1 USD @ 1 EUR")
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
                status: None,
                account_name: AccountName::from_parts(&[
                    String::from("assets"),
                    String::from("bank"),
                    String::from("checking"),
                ]),
                amount: Some(Amount {
                    quantity: Decimal::new(1, 0),
                    commodity: String::from("USD"),
                }),
                price: Some(AmountPrice::Unit(Amount {
                    quantity: Decimal::new(1, 0),
                    commodity: String::from("EUR"),
                })),
                assertion: None,
                is_virtual: false,
            })
        );
    }

    #[test]
    fn virtual_posting() {
        let result = posting()
            .then_ignore(end())
            .parse(" (assets:bank:checking)  $1")
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
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
                is_virtual: true,
            })
        );
    }

    #[test]
    fn not_enough_spaces() {
        let result = posting()
            .then_ignore(end())
            .parse(" assets:bank:checking $1")
            .into_result();
        assert_eq!(
            result,
            Ok(Posting {
                status: None,
                account_name: AccountName::from_parts(&[
                    String::from("assets"),
                    String::from("bank"),
                    String::from("checking $1"),
                ]),
                amount: None,
                price: None,
                assertion: None,
                is_virtual: false,
            })
        );
    }

    #[test]
    fn no_ident() {
        let result = posting()
            .then_ignore(end())
            .parse("assets:bank:checking $1")
            .into_result();
        assert!(result.is_err());
    }
}
