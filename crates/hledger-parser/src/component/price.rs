use chumsky::prelude::*;

use crate::component::amount::{amount, Amount};
use crate::component::whitespace::whitespace;
use crate::state::State;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq)]
pub enum AmountPrice {
    Unit(Amount),
    Total(Amount),
}

#[allow(clippy::module_name_repetitions)]
pub fn amount_price<'a>(
) -> impl Parser<'a, &'a str, AmountPrice, extra::Full<Rich<'a, char>, State, ()>> {
    just("@")
        .repeated()
        .at_least(1)
        .at_most(2)
        .collect::<Vec<_>>()
        .then_ignore(whitespace().repeated())
        .then(amount())
        .map(|(price_type, price)| {
            if price_type.len() == 1 {
                AmountPrice::Unit(price)
            } else {
                AmountPrice::Total(price)
            }
        })
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn total_price() {
        let result = amount_price()
            .then_ignore(end())
            .parse("@@   $1.35")
            .into_result();
        assert_eq!(
            result,
            Ok(AmountPrice::Total(Amount {
                commodity: String::from("$"),
                quantity: Decimal::new(135, 2)
            }))
        );
    }

    #[test]
    fn unit_price() {
        let result = amount_price()
            .then_ignore(end())
            .parse("@   $1.35")
            .into_result();
        assert_eq!(
            result,
            Ok(AmountPrice::Unit(Amount {
                commodity: String::from("$"),
                quantity: Decimal::new(135, 2),
            }))
        );
    }
}
