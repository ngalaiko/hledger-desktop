use chumsky::prelude::*;

use crate::component::amount::{amount, Amount};
use crate::component::commodity::commodity as parse_commodity;
use crate::component::whitespace::whitespace;
use crate::state::State;
use crate::utils::end_of_line;

#[derive(Clone, Debug, PartialEq)]
pub enum Commodity {
    Amount(Amount),
    Commodity(String),
}

pub fn commodity<'a>() -> impl Parser<'a, &'a str, Commodity, extra::Full<Rich<'a, char>, State, ()>>
{
    just("commodity")
        .ignore_then(whitespace().repeated().at_least(1))
        .ignore_then(
            amount()
                .map(Commodity::Amount)
                .or(parse_commodity().map(Commodity::Commodity)),
        )
        .then_ignore(end_of_line())
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn with_symbol() {
        let result = commodity()
            .then_ignore(end())
            .parse("commodity $1000.00")
            .into_result();
        assert_eq!(
            result,
            Ok(Commodity::Amount(Amount {
                commodity: String::from("$"),
                quantity: Decimal::new(100_000, 2),
            }))
        );
    }

    #[test]
    fn no_symbol() {
        let result = commodity()
            .then_ignore(end())
            .parse("commodity 1,000,000.0000")
            .into_result();
        assert_eq!(
            result,
            Ok(Commodity::Amount(Amount {
                commodity: String::new(),
                quantity: Decimal::new(10_000_000_000, 4),
            }))
        );
    }

    #[test]
    fn comment() {
        let result = commodity()
            .then_ignore(end())
            .parse("commodity 1. USD ; with comment")
            .into_result();
        assert_eq!(
            result,
            Ok(Commodity::Amount(Amount {
                commodity: String::from("USD"),
                quantity: Decimal::new(1, 0),
            }))
        );
    }

    #[test]
    fn just_currency() {
        let result = commodity()
            .then_ignore(end())
            .parse("commodity \"AAAA 2023\"  ")
            .into_result();
        assert_eq!(result, Ok(Commodity::Commodity(String::from("AAAA 2023"))));
    }
}
