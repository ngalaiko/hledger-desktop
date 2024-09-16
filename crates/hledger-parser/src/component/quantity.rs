use chumsky::prelude::*;
use rust_decimal::Decimal;

use crate::state::State;

pub fn quantity<'a>() -> impl Parser<'a, &'a str, Decimal, extra::Full<Rich<'a, char>, State, ()>> {
    let digits = any()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .at_least(1)
        .collect::<String>();
    let separator = one_of(",.").map(String::from);
    digits
        .or(separator)
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .try_map(|tokens, span| {
            let mut places = 0_u32;
            let mut mantissa = String::new();
            let mut decimal_separator = None;
            let mut thousands_separator = None;
            let mut last_token_was_separator = false;
            for token in tokens.iter().rev() {
                let is_separator = token == "." || token == ",";
                if is_separator {
                    if last_token_was_separator {
                        return Err(Rich::custom(span, "unexpected separator"));
                    }
                    last_token_was_separator = true;
                    match (decimal_separator, thousands_separator) {
                        (None, None) => {
                            // assume first seen separator is a decimal separator
                            decimal_separator.replace(token);
                        }
                        (Some(sep), None) if token == sep => {
                            // if second separator encountered, and it's the same as decimal
                            // separator, we are in the only thousands_separator handing case.
                            places = 0;
                            decimal_separator = None;
                            thousands_separator.replace(token);
                        }
                        (Some(_), None) => {
                            thousands_separator.replace(token);
                        }
                        (None | Some(_), Some(thousands_separator)) => {
                            if token != thousands_separator {
                                return Err(Rich::custom(
                                    span,
                                    "unexpected thousands separator: {token}",
                                ));
                            };
                        }
                    }
                } else {
                    last_token_was_separator = false;
                    if decimal_separator.is_none() {
                        places += u32::try_from(token.len()).unwrap();
                    }
                    mantissa = token.to_owned() + &mantissa;
                }
            }

            if decimal_separator.is_none() {
                places = 0;
            }

            match mantissa.parse::<i64>() {
                Ok(mantissa) => Ok(Decimal::new(mantissa, places)),
                Err(_) => Err(Rich::custom(span, "failed to parse number")),
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer() {
        let result = quantity().then_ignore(end()).parse("123").into_result();
        assert_eq!(result, Ok(Decimal::new(123, 0)));
    }

    #[test]
    fn integer_trailing() {
        let result1 = quantity().then_ignore(end()).parse("123.").into_result();
        let result2 = quantity().then_ignore(end()).parse("123,").into_result();
        assert_eq!(result1, result2);
        assert_eq!(result2, Ok(Decimal::new(123, 0)));
    }

    #[test]
    fn decimals_leading() {
        let result1 = quantity().then_ignore(end()).parse(".0123").into_result();
        let result2 = quantity().then_ignore(end()).parse(",0123").into_result();
        assert_eq!(result1, result2);
        assert_eq!(result2, Ok(Decimal::new(123, 4)));
    }

    #[test]
    fn decimals_invalid() {
        let result = quantity().then_ignore(end()).parse("1..23").into_result();
        assert!(result.is_err());
    }

    #[test]
    fn decimals() {
        let result1 = quantity().then_ignore(end()).parse("1.2345").into_result();
        let result2 = quantity().then_ignore(end()).parse("1,2345").into_result();
        assert_eq!(result1, result2);
        assert_eq!(result2, Ok(Decimal::new(12345, 4)));
    }

    #[test]
    fn decimals_like_thousands() {
        let result1 = quantity().then_ignore(end()).parse("1.234").into_result();
        let result2 = quantity().then_ignore(end()).parse("1,234").into_result();
        assert_eq!(result1, result2);
        assert_eq!(result2, Ok(Decimal::new(1234, 3)));
    }

    #[test]
    fn thousands_trailing() {
        let result1 = quantity()
            .then_ignore(end())
            .parse("12,345,678.")
            .into_result();
        let result2 = quantity()
            .then_ignore(end())
            .parse("12.345.678,")
            .into_result();
        assert_eq!(result1, result2);
        assert_eq!(result2, Ok(Decimal::new(12_345_678, 0)));
    }

    #[test]
    fn weird_thousands() {
        let result = quantity()
            .then_ignore(end())
            .parse("12.34.678")
            .into_result();
        assert_eq!(result, Ok(Decimal::new(1_234_678, 0)));
    }

    #[test]
    fn thousands() {
        let result1 = quantity()
            .then_ignore(end())
            .parse("12,345,678")
            .into_result();
        let result2 = quantity()
            .then_ignore(end())
            .parse("12.345.678")
            .into_result();
        assert_eq!(result1, result2);
        assert_eq!(result2, Ok(Decimal::new(12_345_678, 0)));
    }

    #[test]
    fn thousands_and_decimals() {
        let result1 = quantity()
            .then_ignore(end())
            .parse("12,345.678")
            .into_result();
        let result2 = quantity()
            .then_ignore(end())
            .parse("12.345,678")
            .into_result();
        assert_eq!(result1, result2);
        assert_eq!(result2, Ok(Decimal::new(12_345_678, 3)));
    }
}
