/// TODO:
/// * support transitive rates
use std::collections::HashMap;

use chrono::NaiveDate;

use crate::hledger::{Commodity, Price, Quantity};

#[derive(Clone)]
struct DateRate(NaiveDate, Quantity);

#[derive(Clone)]
pub struct Converter {
    from_to_rates: HashMap<Commodity, HashMap<Commodity, Vec<DateRate>>>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("no rate found for {from}:{to}")]
    NoRate { from: Commodity, to: Commodity },
}

impl Converter {
    pub fn new(prices: &[Price]) -> Self {
        Self {
            from_to_rates: prices.iter().fold(HashMap::new(), |mut rates, price| {
                // insert forward rate
                rates
                    .entry(price.from.clone())
                    .or_insert_with(HashMap::new)
                    .entry(price.to.clone())
                    .or_insert_with(Vec::new)
                    .push(DateRate(price.date, price.rate.clone()));

                // make sure the latest rate is last
                rates
                    .get_mut(&price.from)
                    .unwrap()
                    .get_mut(&price.to)
                    .unwrap()
                    .sort_by_key(|DateRate(date, _)| *date);

                // insert reverse rate
                rates
                    .entry(price.to.clone())
                    .or_insert_with(HashMap::new)
                    .entry(price.from.clone())
                    .or_insert_with(Vec::new)
                    .push(DateRate(price.date, Quantity::ONE / price.rate.clone()));

                // make sure the latest rate is last
                rates
                    .get_mut(&price.to)
                    .unwrap()
                    .get_mut(&price.from)
                    .unwrap()
                    .sort_by_key(|DateRate(date, _)| *date);

                rates
            }),
        }
    }

    fn find_rates_for_pair(&self, from: &Commodity, to: &Commodity) -> Option<&[DateRate]> {
        self.from_to_rates
            .get(from)
            .and_then(|rates| rates.get(to).map(|rates| &rates[..]))
    }

    pub fn convert(
        &self,
        amount: (&Quantity, &Commodity),
        target: &Commodity,
        target_date: &NaiveDate,
    ) -> Result<Quantity, Error> {
        if amount.1 == target {
            return Ok(amount.0.clone());
        }

        let rates = self
            .find_rates_for_pair(amount.1, target)
            .ok_or(Error::NoRate {
                from: amount.1.clone(),
                to: target.clone(),
            })?;

        let latest_rate = rates
            .iter()
            .filter(|DateRate(date, _)| date <= target_date)
            .last()
            .ok_or(Error::NoRate {
                from: amount.1.clone(),
                to: target.clone(),
            })?;

        let result = amount.0.clone() * latest_rate.1.clone();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::*;

    #[test]
    fn test_convertion() {
        let prices = vec![
            Price {
                date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                from: Commodity::from("EUR"),
                to: Commodity::from("USD"),
                rate: Decimal::from_f64(2.5).unwrap().into(),
            },
            Price {
                date: NaiveDate::from_ymd_opt(2021, 1, 2).unwrap(),
                from: Commodity::from("EUR"),
                to: Commodity::from("USD"),
                rate: Decimal::from_f64(0.2).unwrap().into(),
            },
            Price {
                date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                from: Commodity::from("USD"),
                to: Commodity::from("SEK"),
                rate: Decimal::from_f64(10.0).unwrap().into(),
            },
        ];
        let converter = Converter::new(&prices);
        vec![
            (
                // no conversion
                (
                    &Decimal::from_f64(3.0).unwrap().into(),
                    &Commodity::from("EUR"),
                ),
                Commodity::from("EUR"),
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                Ok(Decimal::from_f64(3.0).unwrap().into()),
            ),
            (
                // forward conversion with latest rate
                (
                    &Decimal::from_f64(3.0).unwrap().into(),
                    &Commodity::from("EUR"),
                ),
                Commodity::from("USD"),
                NaiveDate::from_ymd_opt(2021, 1, 2).unwrap(),
                Ok(Decimal::from_f64(0.6).unwrap().into()),
            ),
            (
                // forward conversion with earliest rate
                (
                    &Decimal::from_f64(3.0).unwrap().into(),
                    &Commodity::from("EUR"),
                ),
                Commodity::from("USD"),
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                Ok(Decimal::from_f64(7.5).unwrap().into()),
            ),
            (
                // reverse conversion with latest rate
                (
                    &Decimal::from_f64(5.2).unwrap().into(),
                    &Commodity::from("USD"),
                ),
                Commodity::from("EUR"),
                NaiveDate::from_ymd_opt(2021, 1, 2).unwrap(),
                Ok(Decimal::from_f64(26.0).unwrap().into()),
            ),
            (
                // reverse conversion with earliest rate
                (
                    &Decimal::from_f64(5.2).unwrap().into(),
                    &Commodity::from("USD"),
                ),
                Commodity::from("EUR"),
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                Ok(Decimal::from_f64(2.08).unwrap().into()),
            ),
            (
                // conversion no rate
                (
                    &Decimal::from_f64(3.0).unwrap().into(),
                    &Commodity::from("EUR"),
                ),
                Commodity::from("GBP"),
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                Err(Error::NoRate {
                    from: Commodity::from("EUR"),
                    to: Commodity::from("GBP"),
                }),
            ),
        ]
        .iter()
        .for_each(|(amount, target, date, expected)| {
            assert_eq!(converter.convert(*amount, target, date), *expected);
        });
    }
}
