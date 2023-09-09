use std::{fmt, ops, path, str::FromStr};

use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use serde::ser::SerializeStruct;
use serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tag(String, String);

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AccountDeclarationInfo {
    #[serde(rename = "adicomment")]
    pub comment: String,
    #[serde(rename = "aditags")]
    pub tags: Vec<Tag>,
    #[serde(rename = "adideclarationorder")]
    pub declaration_order: usize,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Quantity(Decimal);

impl serde::Serialize for Quantity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Quantity", 3)?;
        state.serialize_field("decimalMantissa", &self.0.mantissa())?;
        state.serialize_field("decimalPlaces", &self.0.scale())?;
        state.serialize_field("floatingPoint", &self.0.to_f64().unwrap())?;
        state.end()
    }
}

impl<'d> serde::Deserialize<'d> for Quantity {
    fn deserialize<D: serde::Deserializer<'d>>(deserializer: D) -> Result<Self, D::Error> {
        let value: Value = serde::Deserialize::deserialize(deserializer)?;
        let decimal_mantissa = value["decimalMantissa"]
            .as_i64()
            .ok_or_else(|| serde::de::Error::custom("decimalMantissa is not an integer"))?;
        let decimal_places = value["decimalPlaces"]
            .as_u64()
            .ok_or_else(|| serde::de::Error::custom("decimalPlaces is not an integer"))?;
        Ok(Quantity(Decimal::new(
            decimal_mantissa,
            decimal_places as u32,
        )))
    }
}

impl Quantity {
    pub const ONE: Quantity = Quantity(Decimal::ONE);
}

impl From<Quantity> for Decimal {
    fn from(value: Quantity) -> Self {
        value.0
    }
}

impl From<Decimal> for Quantity {
    fn from(value: Decimal) -> Self {
        Quantity(value)
    }
}

impl ops::Add for Quantity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let self_decimal: Decimal = self.into();
        let rhs_decimal: Decimal = rhs.into();
        let result_decimal = self_decimal + rhs_decimal;
        result_decimal.normalize().into()
    }
}

impl ops::Sub for Quantity {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let self_decimal: Decimal = self.into();
        let rhs_decimal: Decimal = rhs.into();
        let result_decimal = self_decimal - rhs_decimal;
        result_decimal.normalize().into()
    }
}

impl ops::Mul for Quantity {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let self_decimal: Decimal = self.into();
        let rhs_decimal: Decimal = rhs.into();
        let result_decimal = self_decimal * rhs_decimal;
        result_decimal.normalize().into()
    }
}

impl ops::Div for Quantity {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let self_decimal: Decimal = self.into();
        let rhs_decimal: Decimal = rhs.into();
        let result_decimal = self_decimal / rhs_decimal;
        result_decimal.normalize().into()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Side {
    #[serde(rename = "L")]
    Left,
    #[serde(rename = "R")]
    #[default]
    Right,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DigitGroupStyle((char, Vec<usize>));

impl Iterator for DigitGroupStyle {
    type Item = (char, usize);

    // The last group size is assumed to repeat
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 .1.len() == 1 {
            Some((self.0 .0, self.0 .1[0]))
        } else {
            let group_size = self.0 .1.remove(0);
            Some((self.0 .0, group_size))
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AmountStyle {
    #[serde(rename = "ascommodityside")]
    pub commodity_side: Side,
    #[serde(rename = "ascommodityspaced")]
    pub spaced: bool,
    #[serde(rename = "asprecision")]
    pub precision: usize,
    #[serde(rename = "asdecimalpoint")]
    pub decimal_point: Option<char>,
    #[serde(rename = "asdigitgroups")]
    pub digit_groups: Option<DigitGroupStyle>,
}

pub type Commodity = String;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "tag", content = "contents")]
pub enum AmountPrice {
    TotalPrice(Amount),
    UnitPrice(Amount),
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Amount {
    #[serde(rename = "acommodity")]
    pub commodity: Commodity,
    #[serde(rename = "aquantity")]
    pub quantity: Quantity,
    #[serde(rename = "astyle")]
    pub style: AmountStyle,
    #[serde(rename = "aprice")]
    pub price: Option<Box<AmountPrice>>,
}

lazy_static! {
    static ref UNQUOTED_COMMODITY: Regex = Regex::new(
        r"^([^[[:digit:]][[:space:]][-!?\.,\+]]+)|([^[[:digit:]][[:space:]][-!?\.,\+]]+)$"
    )
    .unwrap();
    static ref QUOTED_COMMODITY: Regex = Regex::new(r#"^(".+")|(".+")$"#).unwrap();
}

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum ParseAmountError {
    #[error("failed to parse quantity: {0}")]
    InvalidAmout(String),
    #[error("quantity not found")]
    MissingAmount,
}

impl FromStr for Amount {
    type Err = ParseAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("@@") {
            s.splitn(2, "@@")
                .map(Self::from_str)
                .collect::<Result<Vec<_>, _>>()
                .map(|parsed| Amount {
                    price: Some(Box::new(AmountPrice::TotalPrice(parsed[1].clone()))),
                    ..parsed[0].clone()
                })
        } else if s.contains('@') {
            s.splitn(2, '@')
                .map(Self::from_str)
                .collect::<Result<Vec<_>, _>>()
                .map(|parsed| Amount {
                    price: Some(Box::new(AmountPrice::UnitPrice(parsed[1].clone()))),
                    ..parsed[0].clone()
                })
        } else {
            let s = s.trim();

            // maybe negative sign is before commodity
            let is_negative = s.starts_with('-');
            let s = s.trim_start_matches('-').trim_start_matches('+').trim();

            // first, determine commodity and it's side
            let (side, commodity) = QUOTED_COMMODITY
                .captures(s)
                .and_then(|caps| {
                    caps.get(1)
                        .map(|m| (Side::Left, m.as_str()))
                        .or_else(|| caps.get(2).map(|m| (Side::Right, m.as_str())))
                })
                .or_else(|| {
                    UNQUOTED_COMMODITY.captures(s).and_then(|caps| {
                        caps.get(1)
                            .map(|m| (Side::Left, m.as_str()))
                            .or_else(|| caps.get(2).map(|m| (Side::Right, m.as_str())))
                    })
                })
                .unwrap_or((Side::Right, ""));

            // remove parsed commodity from string
            let s = s.replace(commodity, "");

            if s.is_empty() {
                return Err(ParseAmountError::MissingAmount);
            }

            // determine if commodity is spaced
            let spaced = match side {
                Side::Left => s.chars().next().unwrap().is_whitespace(),
                Side::Right => s.chars().last().unwrap().is_whitespace(),
            };

            // remove spaces from string
            let s = match side {
                Side::Left => s.trim_start(),
                Side::Right => s.trim_end(),
            };

            // maybe negative sign is before digit
            let is_negative = if is_negative {
                is_negative
            } else {
                s.starts_with('-')
            };

            // determine decimal point. it's either the last comma or the last dot
            let decimal_point = s.chars().filter(|c| c.eq(&',') || c.eq(&'.')).last();

            // precision is the number of digits after the decimal point
            let decimal_places = decimal_point.map_or(0, |c| s.split(c).last().unwrap().len());

            let decimal_mantissa = match decimal_point {
                Some(d) => s
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .take_while(|c| !c.eq(&d))
                    .collect::<String>(),
                None => s.chars().filter(|c| c.is_ascii_digit()).collect::<String>(),
            }
            .parse::<i64>()
            .map_err(|_| ParseAmountError::InvalidAmout(s.to_string()))?;

            Ok(Self {
                commodity: commodity.replace('"', "").to_string(),
                quantity: Quantity(if is_negative {
                    Decimal::new(-decimal_mantissa, decimal_places as u32)
                } else {
                    Decimal::new(decimal_mantissa, decimal_places as u32)
                }),
                style: AmountStyle {
                    commodity_side: side,
                    spaced,
                    precision: decimal_places,
                    decimal_point,
                    digit_groups: None,
                },
                price: None,
            })
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_negative = self.quantity.0.is_sign_negative();
        let decimal_mantissa = self.quantity.0.abs().mantissa();

        let integer_part = if let Some(groups) = &self.style.digit_groups {
            let integer_part = decimal_mantissa.to_string();
            let mut integer_part = if integer_part.len() > self.quantity.0.scale() as usize {
                integer_part
                    .chars()
                    .take(integer_part.len() - self.quantity.0.scale() as usize)
                    .collect::<Vec<_>>()
            } else {
                integer_part.chars().collect::<Vec<_>>()
            }
            .into_iter()
            .rev()
            .collect::<String>();

            let mut result: Vec<char> = vec![];
            let mut groups_iter = groups.clone();
            loop {
                let (separator, count) = groups_iter.next().unwrap();

                let head = integer_part
                    .drain(..count.min(integer_part.len()))
                    .collect::<String>();

                result.extend(head.chars());
                if integer_part.is_empty() {
                    break;
                } else {
                    result.push(separator);
                }
            }
            result.into_iter().rev().collect::<String>()
        } else {
            let integer_part = decimal_mantissa.to_string();
            if integer_part.len() > self.quantity.0.scale() as usize {
                integer_part
                    .chars()
                    .take(integer_part.len() - self.quantity.0.scale() as usize)
                    .collect::<String>()
            } else {
                integer_part
            }
        };

        let fractional_part = decimal_mantissa
            .to_string()
            .chars()
            .skip(integer_part.len())
            .collect::<String>();

        let quantity = if self.quantity.0.scale() == 0 {
            format!("{}{}", if is_negative { "-" } else { "" }, integer_part)
        } else {
            format!(
                "{}{}{}{:0>width$}",
                if is_negative { "-" } else { "" },
                integer_part,
                self.style.decimal_point.unwrap_or('.'),
                fractional_part,
                width = self.quantity.0.scale() as usize
            )
        };

        match self.style.commodity_side {
            Side::Left => {
                if self.style.spaced {
                    write!(f, "{} {}", self.commodity, quantity)
                } else {
                    write!(f, "{}{}", self.commodity, quantity)
                }
            }
            Side::Right => {
                if self.style.spaced {
                    write!(f, "{} {}", quantity, self.commodity)
                } else {
                    write!(f, "{}{}", quantity, self.commodity)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantity_serde() {
        let raw = r#"{"decimalMantissa":123456,"decimalPlaces":3,"floatingPoint":123.456}"#;
        let quantity: Quantity = serde_json::from_str(raw).unwrap();
        assert_eq!(quantity.0, Decimal::new(123456, 3));
        let quantity = serde_json::to_string(&quantity).unwrap();
        assert_eq!(quantity, raw);
    }

    #[test]
    fn test_amount_parse() {
        vec![
            ("s", Err(ParseAmountError::MissingAmount)),
            (
                "1",
                Ok(Amount {
                    commodity: "".to_string(),
                    quantity: Quantity(Decimal::new(1, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: false,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "$1",
                Ok(Amount {
                    commodity: "$".to_string(),
                    quantity: Quantity(Decimal::new(1, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Left,
                        spaced: false,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "4000 AAPL",
                Ok(Amount {
                    commodity: "AAPL".to_string(),
                    quantity: Quantity(Decimal::new(4000, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: true,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "3 \"green apples\"",
                Ok(Amount {
                    commodity: "green apples".to_string(),
                    quantity: Quantity(Decimal::new(3, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: true,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "-$1",
                Ok(Amount {
                    commodity: "$".to_string(),
                    quantity: Quantity(Decimal::new(-1, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Left,
                        spaced: false,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "$-1",
                Ok(Amount {
                    commodity: "$".to_string(),
                    quantity: Quantity(Decimal::new(-1, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Left,
                        spaced: false,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "+ $1",
                Ok(Amount {
                    commodity: "$".to_string(),
                    quantity: Quantity(Decimal::new(1, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Left,
                        spaced: false,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "$-      1",
                Ok(Amount {
                    commodity: "$".to_string(),
                    quantity: Quantity(Decimal::new(-1, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Left,
                        spaced: false,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "1.23",
                Ok(Amount {
                    commodity: "".to_string(),
                    quantity: Quantity(Decimal::new(123, 2)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: false,
                        precision: 2,
                        decimal_point: Some('.'),
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "1,23456780000009",
                Ok(Amount {
                    commodity: "".to_string(),
                    quantity: Quantity(Decimal::new(123456780000009, 14)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: false,
                        precision: 14,
                        decimal_point: Some(','),
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "EUR 2.000.000,00",
                Ok(Amount {
                    commodity: "EUR".to_string(),
                    quantity: Quantity(Decimal::new(200000000, 2)),
                    style: AmountStyle {
                        commodity_side: Side::Left,
                        spaced: true,
                        precision: 2,
                        decimal_point: Some(','),
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "INR 9,99,99,999.00",
                Ok(Amount {
                    commodity: "INR".to_string(),
                    quantity: Quantity(Decimal::new(9999999900, 2)),
                    style: AmountStyle {
                        commodity_side: Side::Left,
                        spaced: true,
                        precision: 2,
                        decimal_point: Some('.'),
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "1 000 000.9455",
                Ok(Amount {
                    commodity: "".to_string(),
                    quantity: Quantity(Decimal::new(10000009455, 4)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: false,
                        precision: 4,
                        decimal_point: Some('.'),
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "-2 \"Liquorice Wands\"",
                Ok(Amount {
                    commodity: "Liquorice Wands".to_string(),
                    quantity: Quantity(Decimal::new(-2, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: true,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                }),
            ),
            (
                "1 SEK @ 1.2 USD",
                Ok(Amount {
                    commodity: "SEK".to_string(),
                    quantity: Quantity(Decimal::new(1, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: true,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: Some(Box::new(AmountPrice::UnitPrice(Amount {
                        commodity: "USD".to_string(),
                        quantity: Quantity(Decimal::new(12, 1)),
                        style: AmountStyle {
                            commodity_side: Side::Right,
                            spaced: true,
                            precision: 1,
                            decimal_point: Some('.'),
                            digit_groups: None,
                        },
                        price: None,
                    }))),
                }),
            ),
            (
                "1 SEK @@ 1.2 USD",
                Ok(Amount {
                    commodity: "SEK".to_string(),
                    quantity: Quantity(Decimal::new(1, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: true,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: Some(Box::new(AmountPrice::TotalPrice(Amount {
                        commodity: "USD".to_string(),
                        quantity: Quantity(Decimal::new(12, 1)),
                        style: AmountStyle {
                            commodity_side: Side::Right,
                            spaced: true,
                            precision: 1,
                            decimal_point: Some('.'),
                            digit_groups: None,
                        },
                        price: None,
                    }))),
                }),
            ),
        ]
        .into_iter()
        .for_each(|(raw, expected)| {
            assert_eq!(raw.parse::<Amount>(), expected, "failed to parse {}", raw);
        });
    }

    #[test]
    fn test_amount_display() {
        vec![
            (
                Amount {
                    commodity: "SEK".to_string(),
                    quantity: Quantity(Decimal::new(1200000, 2)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: true,
                        precision: 2,
                        decimal_point: Some('.'),
                        digit_groups: Some(DigitGroupStyle((',', vec![3]))),
                    },
                    price: None,
                },
                "12,000.00 SEK",
            ),
            (
                Amount {
                    commodity: "SEK".to_string(),
                    quantity: Quantity(Decimal::new(-100, 0)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: true,
                        precision: 0,
                        decimal_point: None,
                        digit_groups: None,
                    },
                    price: None,
                },
                "-100 SEK",
            ),
            (
                Amount {
                    commodity: "SEK".to_string(),
                    quantity: Quantity(Decimal::new(-1200000, 2)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: true,
                        precision: 2,
                        decimal_point: Some('.'),
                        digit_groups: Some(DigitGroupStyle((',', vec![3]))),
                    },
                    price: None,
                },
                "-12,000.00 SEK",
            ),
            (
                Amount {
                    commodity: "SEK".to_string(),
                    quantity: Quantity(Decimal::new(-30000, 2)),
                    style: AmountStyle {
                        commodity_side: Side::Right,
                        spaced: true,
                        precision: 2,
                        decimal_point: Some('.'),
                        digit_groups: Some(DigitGroupStyle((',', vec![3]))),
                    },
                    price: None,
                },
                "-300.00 SEK",
            ),
        ]
        .into_iter()
        .for_each(|(amount, expected)| {
            assert_eq!(format!("{}", amount), expected);
        });
    }
}

pub type MixedAmount = Vec<Amount>;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct AccountName(String);

impl AccountName {
    pub fn basename(&self) -> &str {
        self.0.split(':').last().unwrap()
    }

    pub fn depth(&self) -> usize {
        if self.0.is_empty() {
            0
        } else {
            self.0.split(':').count()
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseAccountNameError {
    #[error("must not be empty")]
    Empty,
}

impl FromStr for AccountName {
    type Err = ParseAccountNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseAccountNameError::Empty);
        }
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for AccountName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Account {
    #[serde(rename = "aname")]
    pub name: AccountName,
    #[serde(rename = "adeclarationinfo")]
    pub declaration_info: Option<AccountDeclarationInfo>,
    #[serde(rename = "asubs_")]
    pub subaccounts: Vec<AccountName>,
    #[serde(rename = "aparent_")]
    pub parent: AccountName,
    #[serde(rename = "aboring")]
    pub boring: bool,
    #[serde(rename = "anumpostings")]
    pub num_postings: usize,
    #[serde(rename = "aebalance")]
    pub balance_excluding_subsaccounts: MixedAmount,
    #[serde(rename = "aibalance")]
    pub balance_including_subsaccounts: MixedAmount,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SourcePosition {
    #[serde(rename = "sourceColumn")]
    pub column: usize,
    #[serde(rename = "sourceLine")]
    pub line: usize,
    #[serde(rename = "sourceName")]
    pub file_name: path::PathBuf,
}

pub type SourceRange = (SourcePosition, SourcePosition);

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum Status {
    #[default]
    Unmarked,
    Pending,
    Cleared,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    #[serde(rename = "tindex")]
    pub index: usize,
    #[serde(rename = "tprecedingcomment")]
    pub preceding_comment: String,
    #[serde(rename = "tsourcepos")]
    pub source_position: SourceRange,
    #[serde(rename = "tdate")]
    pub date: chrono::NaiveDate,
    #[serde(rename = "tdate2")]
    pub date2: Option<chrono::NaiveDate>,
    #[serde(rename = "tstatus")]
    pub status: Status,
    #[serde(rename = "tcode")]
    pub code: String,
    #[serde(rename = "tdescription")]
    pub description: String,
    #[serde(rename = "tcomment")]
    pub comment: String,
    #[serde(rename = "ttags")]
    pub tags: Vec<Tag>,
    #[serde(rename = "tpostings")]
    pub postings: Vec<Posting>,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum PostingType {
    #[serde(rename = "RegularPosting")]
    #[default]
    Regular,
    #[serde(rename = "VirtualPosting")]
    Virtual,
    #[serde(rename = "BalancedVirtualPosting")]
    BalancedVirtual,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BalanceAssertion {
    #[serde(rename = "baamount")]
    pub amount: Amount,
    #[serde(rename = "batotal")]
    pub total: bool,
    #[serde(rename = "bainclusive")]
    pub inclusive: bool,
    #[serde(rename = "baposition")]
    pub position: SourcePosition,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Posting {
    #[serde(rename = "pdate")]
    pub date: Option<chrono::NaiveDate>,
    #[serde(rename = "pdate2")]
    pub date2: Option<chrono::NaiveDate>,
    #[serde(rename = "pstatus")]
    pub status: Status,
    #[serde(rename = "paccount")]
    pub account: AccountName,
    #[serde(rename = "pamount")]
    pub amount: MixedAmount,
    #[serde(rename = "pcomment")]
    pub comment: String,
    #[serde(rename = "ptype")]
    pub posting_type: PostingType,
    #[serde(rename = "ptags")]
    pub tags: Vec<Tag>,
    #[serde(rename = "pbalanceassertion")]
    pub balance_assertion: Option<BalanceAssertion>,
    #[serde(rename = "ptransaction")]
    pub transaction: Option<usize>,
    #[serde(rename = "poriginal")]
    pub original: Option<Box<Posting>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Price {
    #[serde(rename = "mpdate")]
    pub date: chrono::NaiveDate,
    #[serde(rename = "mpfrom")]
    pub from: Commodity,
    #[serde(rename = "mpto")]
    pub to: Commodity,
    #[serde(rename = "mprate")]
    pub rate: Quantity,
}
