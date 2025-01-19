use chumsky::prelude::*;
use rust_decimal::Decimal;

use crate::component::commodity::commodity;
use crate::component::quantity::quantity;
use crate::component::whitespace::whitespace;
use crate::state::State;
use crate::AmountPrice;

#[derive(Debug, Default, Clone, Hash, PartialEq)]
pub struct Amount {
    pub quantity: Decimal,
    pub commodity: String,
    pub price: Option<Box<AmountPrice>>,
}

impl Amount {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn cost(&self) -> Option<Amount> {
        match self.price.as_ref() {
            None => None,
            Some(price) => match price.as_ref() {
                crate::AmountPrice::Total(price) if self.quantity.is_sign_positive() => {
                    Some(price.clone())
                }
                crate::AmountPrice::Total(price) => {
                    let mut price = price.clone();
                    price.quantity.set_sign_negative(true);
                    Some(price)
                }
                crate::AmountPrice::Unit(price) => Some(Amount {
                    quantity: price.quantity.checked_mul(self.quantity).expect("overflow"),
                    commodity: price.commodity.clone(),
                    price: None,
                }),
            },
        }
    }
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
                price: None,
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
                price: None,
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
                price: None,
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
                price: None,
            }
        });
    let quantity_commodity = quantity()
        .then_ignore(whitespace().repeated())
        .then(commodity())
        .map(|(quantity, commodity)| Amount {
            quantity,
            commodity,
            price: None,
        });
    let commodity_quantity = commodity()
        .then_ignore(whitespace().repeated())
        .then(quantity())
        .map(|(commodity, quantity)| Amount {
            quantity,
            commodity,
            price: None,
        });
    let just_quantity = quantity().map(|quantity| Amount {
        quantity,
        commodity: String::new(),
        price: None,
    });
    choice((
        sign_quantity_commodity.boxed(),
        quantity_sign_commodity.boxed(),
        sign_commodity_quantity.boxed(),
        commodity_sign_quantity.boxed(),
        quantity_commodity.boxed(),
        commodity_quantity.boxed(),
        just_quantity.boxed(),
    ))
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
                    price: None,
                },
            ),
            (
                "4000 AAPL",
                Amount {
                    quantity: Decimal::new(4000, 0),
                    commodity: String::from("AAPL"),
                    price: None,
                },
            ),
            (
                "3 \"green apples\"",
                Amount {
                    quantity: Decimal::new(3, 0),
                    commodity: String::from("green apples"),
                    price: None,
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
                    price: None,
                },
            ),
            (
                "$-1",
                Amount {
                    quantity: Decimal::new(-1, 0),
                    commodity: String::from("$"),
                    price: None,
                },
            ),
            (
                "+ $1",
                Amount {
                    quantity: Decimal::new(1, 0),
                    commodity: String::from("$"),
                    price: None,
                },
            ),
            (
                "$-      1",
                Amount {
                    quantity: Decimal::new(-1, 0),
                    commodity: String::from("$"),
                    price: None,
                },
            ),
            (
                "-1 USD",
                Amount {
                    quantity: Decimal::new(-1, 0),
                    commodity: String::from("USD"),
                    price: None,
                },
            ),
        ] {
            let result = amount().then_ignore(end()).parse(input).into_result();
            assert_eq!(result, Ok(expected), "{input}");
        }
    }
}
