use chumsky::prelude::*;

use crate::component::amount::{amount, Amount};
use crate::component::commodity::commodity;
use crate::component::date::simple::date;
use crate::component::time::time;
use crate::component::whitespace::whitespace;
use crate::state::State;
use crate::utils::end_of_line;

#[derive(Clone, Debug, PartialEq)]
pub struct Price {
    pub date: chrono::NaiveDate,
    pub commodity: String,
    pub amount: Amount,
}

pub fn price<'a>() -> impl Parser<'a, &'a str, Price, extra::Full<Rich<'a, char>, State, ()>> {
    just("P")
        .ignore_then(whitespace().repeated().at_least(1))
        .ignore_then(date())
        .then_ignore(whitespace().repeated().at_least(1))
        .then_ignore(time().then(whitespace().repeated().at_least(1)).or_not())
        .then(commodity())
        .then_ignore(whitespace().repeated().at_least(1))
        .then(amount())
        .then_ignore(end_of_line())
        .map(|((date, commodity), amount)| Price {
            date,
            commodity,
            amount,
        })
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn simple() {
        let result = price()
            .then_ignore(end())
            .parse("P 2009-01-01 € $1.35")
            .into_result();
        assert_eq!(
            result,
            Ok(Price {
                date: chrono::NaiveDate::from_ymd_opt(2009, 1, 1).unwrap(),
                commodity: String::from("€"),
                amount: Amount {
                    quantity: Decimal::new(135, 2),
                    commodity: String::from("$"),
                    price: None,
                },
            })
        );
    }

    #[test]
    fn with_time() {
        let result = price()
            .then_ignore(end())
            .parse("P 2024-04-18 00:00:00 BTC 691747.70790400 SEK")
            .into_result();
        assert_eq!(
            result,
            Ok(Price {
                date: chrono::NaiveDate::from_ymd_opt(2024, 4, 18).unwrap(),
                commodity: String::from("BTC"),
                amount: Amount {
                    quantity: Decimal::new(69_174_770_790_400, 8),
                    commodity: String::from("SEK"),
                    price: None,
                },
            })
        );
    }

    #[test]
    fn comment() {
        let result = price()
            .then_ignore(end())
            .parse("P 2009-01-01 € $1.35  ; with comment")
            .into_result();
        assert_eq!(
            result,
            Ok(Price {
                date: chrono::NaiveDate::from_ymd_opt(2009, 1, 1).unwrap(),
                commodity: String::from("€"),
                amount: Amount {
                    quantity: Decimal::new(135, 2),
                    commodity: String::from("$"),
                    price: None,
                },
            })
        );
    }
}
