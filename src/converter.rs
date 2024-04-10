/// TODO:
/// * support transitive rates
use std::collections::HashMap;

use chrono::NaiveDate;

use crate::hledger::{Amount, AmountPrice, AmountStyle, Commodity, Price, Quantity, Transaction};

#[derive(Clone)]
struct DateRate(NaiveDate, Quantity);

#[derive(Clone)]
pub struct Converter {
    rates: HashMap<Commodity, HashMap<Commodity, Vec<DateRate>>>,
    styles: HashMap<Commodity, AmountStyle>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("no rate found for {from}:{to}")]
    NoRate { from: Commodity, to: Commodity },
}

impl Converter {
    pub fn new(prices: &[Price], transactions: &[Transaction]) -> Self {
        Self {
            styles: transactions
                .iter()
                .flat_map(|tx| tx.postings.iter())
                .flat_map(|posting| posting.amount.iter())
                .fold(HashMap::new(), |mut styles, amount| {
                    match amount.price.as_ref() {
                        Some(AmountPrice::TotalPrice(price) | AmountPrice::UnitPrice(price)) => {
                            styles.insert(amount.commodity.clone(), price.style.clone());
                        }
                        None => {}
                    };
                    if !styles.contains_key(&amount.commodity) {
                        styles.insert(amount.commodity.clone(), amount.style.clone());
                    }
                    styles
                }),
            rates: prices.iter().fold(HashMap::new(), |mut rates, price| {
                // insert forward rate
                rates
                    .entry(price.from.clone())
                    .or_default()
                    .entry(price.to.clone())
                    .or_default()
                    .push(DateRate(price.date, price.rate));

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
                    .or_default()
                    .entry(price.from.clone())
                    .or_default()
                    .push(DateRate(price.date, Quantity::ONE / price.rate));

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
        self.rates
            .get(from)
            .and_then(|rates| rates.get(to).map(|rates| &rates[..]))
    }

    pub fn convert(
        &self,
        amount: &Amount,
        target: &Commodity,
        target_date: NaiveDate,
    ) -> Result<Amount, Error> {
        if amount.commodity == *target {
            return Ok(amount.clone());
        }

        let rates = self
            .find_rates_for_pair(&amount.commodity, target)
            .ok_or(Error::NoRate {
                from: amount.commodity.clone(),
                to: target.clone(),
            })?;

        let latest_rate = rates
            .iter()
            .filter(|DateRate(date, _)| *date <= target_date)
            .last()
            .ok_or(Error::NoRate {
                from: amount.commodity.clone(),
                to: target.clone(),
            })?;

        let style = self
            .styles
            .get(target)
            .unwrap_or(&AmountStyle::default())
            .clone();

        Ok(Amount {
            quantity: amount.quantity * latest_rate.1,
            commodity: target.clone(),
            style,
            ..amount.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::*;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn convertion() {
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
        let converter = Converter::new(&prices, &[]);
        for (amount, target, date, expected) in [
            (
                // no conversion
                Amount {
                    quantity: Decimal::from_f64(3.0).unwrap().into(),
                    commodity: Commodity::from("EUR"),
                    ..Default::default()
                },
                Commodity::from("EUR"),
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                Ok(Amount {
                    quantity: Decimal::from_f64(3.0).unwrap().into(),
                    commodity: Commodity::from("EUR"),
                    ..Default::default()
                }),
            ),
            (
                // forward conversion with latest rate
                Amount {
                    quantity: Decimal::from_f64(3.0).unwrap().into(),
                    commodity: Commodity::from("EUR"),
                    ..Default::default()
                },
                Commodity::from("USD"),
                NaiveDate::from_ymd_opt(2021, 1, 2).unwrap(),
                Ok(Amount {
                    quantity: Decimal::from_f64(0.6).unwrap().into(),
                    commodity: Commodity::from("USD"),
                    ..Default::default()
                }),
            ),
            (
                // forward conversion with earliest rate
                Amount {
                    quantity: Decimal::from_f64(3.0).unwrap().into(),
                    commodity: Commodity::from("EUR"),
                    ..Default::default()
                },
                Commodity::from("USD"),
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                Ok(Amount {
                    quantity: Decimal::from_f64(7.5).unwrap().into(),
                    commodity: Commodity::from("USD"),
                    ..Default::default()
                }),
            ),
            (
                // reverse conversion with latest rate
                Amount {
                    quantity: Decimal::from_f64(5.2).unwrap().into(),
                    commodity: Commodity::from("USD"),
                    ..Default::default()
                },
                Commodity::from("EUR"),
                NaiveDate::from_ymd_opt(2021, 1, 2).unwrap(),
                Ok(Amount {
                    quantity: Decimal::from_f64(26.0).unwrap().into(),
                    commodity: Commodity::from("EUR"),
                    ..Default::default()
                }),
            ),
            (
                // reverse conversion with earliest rate
                Amount {
                    quantity: Decimal::from_f64(5.2).unwrap().into(),
                    commodity: Commodity::from("USD"),
                    ..Default::default()
                },
                Commodity::from("EUR"),
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                Ok(Amount {
                    quantity: Decimal::from_f64(2.08).unwrap().into(),
                    commodity: Commodity::from("EUR"),
                    ..Default::default()
                }),
            ),
            (
                // conversion no rate
                Amount {
                    quantity: Decimal::from_f64(3.0).unwrap().into(),
                    commodity: Commodity::from("EUR"),
                    ..Default::default()
                },
                Commodity::from("GBP"),
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                Err(Error::NoRate {
                    from: Commodity::from("EUR"),
                    to: Commodity::from("GBP"),
                }),
            ),
        ] {
            assert_eq!(converter.convert(&amount, &target, date), expected);
        }
    }
}
