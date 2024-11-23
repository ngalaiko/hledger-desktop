use chumsky::prelude::*;
use rust_decimal::Decimal;

use crate::component::commodity::commodity;
use crate::component::quantity::quantity;
use crate::component::whitespace::whitespace;
use crate::state::State;

#[derive(Debug, Default, Clone, Hash, PartialEq)]
pub struct Amount {
    pub quantity: Decimal,
    pub commodity: String,
}

pub fn amount<'a>() -> impl Parser<'a, &'a str, Amount, extra::Full<Rich<'a, char>, State, ()>> {
    let sign_quantity_commodity = one_of("-+")
        .then_ignore(whitespace().repeated())
        .then(quantity())
        .then_ignore(whitespace().repeated())
        .then(commodity())
        .map(|((sign, mut quantity), commodity)| {
            if sign == '-' {
                quantity.set_sign_negative(true);
            }
            Amount {
                quantity,
                commodity,
            }
        });
    let quantity_sign_commodity = quantity()
        .then_ignore(whitespace().repeated())
        .then(one_of("-+"))
        .then_ignore(whitespace().repeated())
        .then(commodity())
        .map(|((mut quantity, sign), commodity)| {
            if sign == '-' {
                quantity.set_sign_negative(true);
            }
            Amount {
                quantity,
                commodity,
            }
        });
    let sign_commodity_quantity = one_of("-+")
        .then_ignore(whitespace().repeated())
        .then(commodity())
        .then_ignore(whitespace().repeated())
        .then(quantity())
        .map(|((sign, commodity), mut quantity)| {
            if sign == '-' {
                quantity.set_sign_negative(true);
            }
            Amount {
                quantity,
                commodity,
            }
        });
    let commodity_sign_quantity = commodity()
        .then_ignore(whitespace().repeated())
        .then(one_of("-+"))
        .then_ignore(whitespace().repeated())
        .then(quantity())
        .map(|((commodity, sign), mut quantity)| {
            if sign == '-' {
                quantity.set_sign_negative(true);
            }
            Amount {
                quantity,
                commodity,
            }
        });
    let quantity_commodity = quantity()
        .then_ignore(whitespace().repeated())
        .then(commodity())
        .map(|(quantity, commodity)| Amount {
            quantity,
            commodity,
        });
    let commodity_quantity = commodity()
        .then_ignore(whitespace().repeated())
        .then(quantity())
        .map(|(commodity, quantity)| Amount {
            quantity,
            commodity,
        });
    let just_quantity = quantity().map(|quantity| Amount {
        quantity,
        ..Amount::default()
    });
    sign_quantity_commodity
        .or(quantity_sign_commodity)
        .or(sign_commodity_quantity)
        .or(commodity_sign_quantity)
        .or(quantity_commodity)
        .or(commodity_quantity)
        .or(just_quantity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantity_no_commodity() {
        let result = amount().then_ignore(end()).parse("1").into_result();
        assert_eq!(
            result,
            Ok(Amount {
                quantity: Decimal::new(1, 0),
                ..Amount::default()
            })
        );
    }

    #[test]
    fn quantity_with_commodity() {
        for (input, expected) in [
            (
                "$1",
                Amount {
                    quantity: Decimal::new(1, 0),
                    commodity: String::from("$"),
                },
            ),
            (
                "4000 AAPL",
                Amount {
                    quantity: Decimal::new(4000, 0),
                    commodity: String::from("AAPL"),
                },
            ),
            (
                "3 \"green apples\"",
                Amount {
                    quantity: Decimal::new(3, 0),
                    commodity: String::from("green apples"),
                },
            ),
        ] {
            let result = amount().then_ignore(end()).parse(input).into_result();
            assert_eq!(result, Ok(expected), "{input}");
        }
    }

    #[test]
    fn signed_quantity_with_commodity() {
        for (input, expected) in [
            (
                "-$1",
                Amount {
                    quantity: Decimal::new(-1, 0),
                    commodity: String::from("$"),
                },
            ),
            (
                "$-1",
                Amount {
                    quantity: Decimal::new(-1, 0),
                    commodity: String::from("$"),
                },
            ),
            (
                "+ $1",
                Amount {
                    quantity: Decimal::new(1, 0),
                    commodity: String::from("$"),
                },
            ),
            (
                "$-      1",
                Amount {
                    quantity: Decimal::new(-1, 0),
                    commodity: String::from("$"),
                },
            ),
            (
                "-1 USD",
                Amount {
                    quantity: Decimal::new(-1, 0),
                    commodity: String::from("USD"),
                },
            ),
        ] {
            let result = amount().then_ignore(end()).parse(input).into_result();
            assert_eq!(result, Ok(expected), "{input}");
        }
    }
}
