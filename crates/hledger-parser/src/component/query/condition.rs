use chumsky::prelude::*;
use rust_decimal::Decimal;

use crate::component::period::period;
use crate::component::quantity::quantity;
use crate::component::status::{status, Status};
use crate::state::State;
use crate::Period;

use crate::component::whitespace::whitespace;

#[derive(Clone, Debug, PartialEq)]
pub enum Condition {
    Account(String),
    Code(String),
    Currency(String),
    Description(String),
    Payee(String),
    Amount(Amount),
    Date(Period),
    Status(Option<Status>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Sign {
    Plus,
    Minus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Amount {
    Equal(Option<Sign>, Decimal),
    Less(Option<Sign>, Decimal),
    LessOrEqual(Option<Sign>, Decimal),
    Greater(Option<Sign>, Decimal),
    GreaterOrEqual(Option<Sign>, Decimal),
}

pub fn condition<'a>() -> impl Parser<'a, &'a str, Condition, extra::Full<Rich<'a, char>, State, ()>>
{
    let account_prefixed = just("acct:")
        .ignore_then(string_value())
        .map(Condition::Account);
    let code = just("code:")
        .ignore_then(string_value())
        .map(Condition::Code);
    let currency = just("cur:")
        .ignore_then(string_value())
        .map(Condition::Currency);
    let description = just("desc:")
        .ignore_then(string_value())
        .map(Condition::Description);
    let payee = just("payee:")
        .ignore_then(string_value())
        .map(Condition::Payee);
    let status = just("status:")
        .ignore_then(status().or_not())
        .map(Condition::Status);
    let amount = amount_condition().map(Condition::Amount);
    let date = just("date:").ignore_then(period()).map(Condition::Date);

    status
        .or(amount)
        .or(date)
        .or(code)
        .or(currency)
        .or(description)
        .or(payee)
        .or(account_prefixed)
        .or(string_value().map(Condition::Account))
}

fn string_value<'a>() -> impl Parser<'a, &'a str, String, extra::Full<Rich<'a, char>, State, ()>> {
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
    quoted_value.or(value)
}

fn amount_condition<'a>() -> impl Parser<'a, &'a str, Amount, extra::Full<Rich<'a, char>, State, ()>>
{
    let equal = sign()
        .or_not()
        .then(quantity())
        .map(|(sign, quantity)| Amount::Equal(sign, quantity));
    let less = just("<")
        .ignore_then(sign().or_not().then(quantity()))
        .map(|(sign, quantity)| Amount::Less(sign, quantity));
    let less_or_equal = just("<=")
        .ignore_then(sign().or_not().then(quantity()))
        .map(|(sign, quantity)| Amount::LessOrEqual(sign, quantity));
    let greater = just(">")
        .ignore_then(sign().or_not().then(quantity()))
        .map(|(sign, quantity)| Amount::Greater(sign, quantity));
    let greater_or_equal = just(">=")
        .ignore_then(sign().or_not().then(quantity()))
        .map(|(sign, quantity)| Amount::GreaterOrEqual(sign, quantity));

    just("amt:").ignore_then(
        less_or_equal
            .or(less)
            .or(greater_or_equal)
            .or(greater)
            .delimited_by(just("'"), just("'"))
            .or(equal),
    )
}

fn sign<'a>() -> impl Parser<'a, &'a str, Sign, extra::Full<Rich<'a, char>, State, ()>> {
    choice([just("+").to(Sign::Plus), just("-").to(Sign::Minus)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amount_equal() {
        let result = condition().then_ignore(end()).parse("amt:12").into_result();
        assert_eq!(
            result,
            Ok(Condition::Amount(Amount::Equal(None, Decimal::from(12))))
        );
    }

    #[test]
    fn amount_less() {
        let result = condition()
            .then_ignore(end())
            .parse("amt:'<12'")
            .into_result();
        assert_eq!(
            result,
            Ok(Condition::Amount(Amount::Less(None, Decimal::from(12))))
        );
    }

    #[test]
    fn amount_less_or_equal() {
        let result = condition()
            .then_ignore(end())
            .parse("amt:'<=-12'")
            .into_result();
        assert_eq!(
            result,
            Ok(Condition::Amount(Amount::LessOrEqual(
                Some(Sign::Minus),
                Decimal::from(12)
            )))
        );
    }

    #[test]
    fn amount_greater() {
        let result = condition()
            .then_ignore(end())
            .parse("amt:'>12'")
            .into_result();
        assert_eq!(
            result,
            Ok(Condition::Amount(Amount::Greater(None, Decimal::from(12))))
        );
    }

    #[test]
    fn amount_greater_or_equal() {
        let result = condition()
            .then_ignore(end())
            .parse("amt:'>=+12'")
            .into_result();
        assert_eq!(
            result,
            Ok(Condition::Amount(Amount::GreaterOrEqual(
                Some(Sign::Plus),
                Decimal::from(12)
            )))
        );
    }

    #[test]
    fn status_pending() {
        let result = condition()
            .then_ignore(end())
            .parse("status:!")
            .into_result();
        assert_eq!(result, Ok(Condition::Status(Some(Status::Pending))));
    }

    #[test]
    fn status_cleared() {
        let result = condition()
            .then_ignore(end())
            .parse("status:*")
            .into_result();
        assert_eq!(result, Ok(Condition::Status(Some(Status::Cleared))));
    }

    #[test]
    fn status_none() {
        let result = condition()
            .then_ignore(end())
            .parse("status:")
            .into_result();
        assert_eq!(result, Ok(Condition::Status(None)));
    }

    #[test]
    fn date() {
        let result = condition()
            .then_ignore(end())
            .parse("date:2016")
            .into_result();
        assert_eq!(
            result,
            Ok(Condition::Date(Period {
                interval: None,
                begin: chrono::NaiveDate::from_ymd_opt(2016, 1, 1),
                end: chrono::NaiveDate::from_ymd_opt(2017, 1, 1),
            }))
        );
    }

    #[test]
    fn account_no_prefix() {
        let result = condition()
            .then_ignore(end())
            .parse("account")
            .into_result();
        assert_eq!(result, Ok(Condition::Account(String::from("account"))));
    }

    #[test]
    fn account_quoted() {
        let result = condition()
            .then_ignore(end())
            .parse("acct:'another account'")
            .into_result();
        assert_eq!(
            result,
            Ok(Condition::Account(String::from("another account")))
        );
    }
}
