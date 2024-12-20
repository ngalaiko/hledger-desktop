use std::time::SystemTime;

use chrono::Datelike;
use chumsky::prelude::*;

use crate::{component::whitespace::whitespace, state::State};

pub fn date<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    choice((
        periods_ahead(),
        periods_ago(),
        n_periods(),
        rel_word_period(),
        words(),
        rel_word_period(),
        month_day(),
        year_month_day(),
        start_of_month_numeric(),
        eight_digits(),
        six_digits(),
        four_digits(),
        just_day(),
        month_name(),
    ))
}

#[derive(Debug, Clone)]
enum Period {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

fn period<'a>() -> impl Parser<'a, &'a str, Period, extra::Full<Rich<'a, char>, State, ()>> {
    choice([
        just("day").to(Period::Day),
        just("week").to(Period::Week),
        just("month").to(Period::Month),
        just("quarter").to(Period::Quarter),
        just("year").to(Period::Year),
    ])
    .then_ignore(just("s").or_not())
}

// n days/weeks/months/quarters/years ago : -n periods from the current period
fn periods_ago<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    text::int(10)
        .then_ignore(whitespace().repeated().at_least(1))
        .then(period())
        .then_ignore(whitespace().repeated().at_least(1))
        .then_ignore(just("ago"))
        .try_map(|(length, period), span| {
            match period {
                Period::Day => {
                    today().checked_sub_days(chrono::Days::new(length.parse::<u64>().unwrap()))
                }
                Period::Week => {
                    today().checked_sub_days(chrono::Days::new(length.parse::<u64>().unwrap() * 7))
                }
                Period::Month => {
                    today().checked_sub_months(chrono::Months::new(length.parse::<u32>().unwrap()))
                }
                Period::Quarter => today()
                    .checked_sub_months(chrono::Months::new(length.parse::<u32>().unwrap() * 3)),
                Period::Year => today()
                    .checked_sub_months(chrono::Months::new(length.parse::<u32>().unwrap() * 12)),
            }
            .ok_or(Rich::custom(span, "not a valid date"))
        })
}

// n days/weeks/months/quarters/years ahead : n periods from the current period
fn periods_ahead<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    text::int(10)
        .then_ignore(whitespace().repeated().at_least(1))
        .then(period())
        .then_ignore(whitespace().repeated().at_least(1))
        .then_ignore(just("ahead"))
        .try_map(|(length, period), span| {
            match period {
                Period::Day => {
                    today().checked_add_days(chrono::Days::new(length.parse::<u64>().unwrap()))
                }
                Period::Week => {
                    today().checked_add_days(chrono::Days::new(length.parse::<u64>().unwrap() * 7))
                }
                Period::Month => {
                    today().checked_add_months(chrono::Months::new(length.parse::<u32>().unwrap()))
                }
                Period::Quarter => today()
                    .checked_add_months(chrono::Months::new(length.parse::<u32>().unwrap() * 3)),
                Period::Year => today()
                    .checked_add_months(chrono::Months::new(length.parse::<u32>().unwrap() * 12)),
            }
            .ok_or(Rich::custom(span, "not a valid date"))
        })
}

// in n days/weeks/months/quarters/years : n periods from the current period
fn n_periods<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    just("in")
        .ignore_then(whitespace().repeated().at_least(1))
        .ignore_then(text::int(10))
        .then_ignore(whitespace().repeated().at_least(1))
        .then(period())
        .try_map(|(length, period), span| {
            match period {
                Period::Day => {
                    today().checked_add_days(chrono::Days::new(length.parse::<u64>().unwrap()))
                }
                Period::Week => {
                    today().checked_add_days(chrono::Days::new(length.parse::<u64>().unwrap() * 7))
                }
                Period::Month => {
                    today().checked_add_months(chrono::Months::new(length.parse::<u32>().unwrap()))
                }
                Period::Quarter => today()
                    .checked_add_months(chrono::Months::new(length.parse::<u32>().unwrap() * 3)),
                Period::Year => today()
                    .checked_add_months(chrono::Months::new(length.parse::<u32>().unwrap() * 12)),
            }
            .ok_or(Rich::custom(span, "not a valid date"))
        })
}

// last/this/next day/week/month/quarter/year : -1, 0, 1 periods from the current period
fn rel_word_period<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    choice([just("last").to(-1), just("this").to(0), just("next").to(1)])
        .then_ignore(whitespace().repeated().at_least(1))
        .then(period())
        .try_map(|(rel, period), span| {
            match period {
                Period::Day => {
                    if rel >= 0 {
                        today().checked_add_days(chrono::Days::new(1))
                    } else {
                        today().checked_sub_days(chrono::Days::new(1))
                    }
                }
                Period::Week => {
                    if rel >= 0 {
                        today().checked_add_days(chrono::Days::new(7))
                    } else {
                        today().checked_sub_days(chrono::Days::new(7))
                    }
                }
                Period::Month => {
                    if rel >= 0 {
                        today().checked_add_months(chrono::Months::new(1))
                    } else {
                        today().checked_sub_months(chrono::Months::new(1))
                    }
                }
                Period::Quarter => {
                    if rel >= 0 {
                        today().checked_add_months(chrono::Months::new(3))
                    } else {
                        today().checked_sub_months(chrono::Months::new(3))
                    }
                }
                Period::Year => {
                    if rel >= 0 {
                        today().checked_add_months(chrono::Months::new(12))
                    } else {
                        today().checked_sub_months(chrono::Months::new(12))
                    }
                }
            }
            .ok_or(Rich::custom(span, "not a valid date"))
        })
}

// yesterday, today, tomorrow : -1, 0, 1 days from today
fn words<'a>() -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>>
{
    choice([
        just("yesterday").to(today().checked_sub_days(chrono::Days::new(1)).unwrap()),
        just("today").to(today()),
        just("tomorrow").to(today().checked_add_days(chrono::Days::new(1)).unwrap()),
    ])
}

// 2024/10/1: exact date
fn year_month_day<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    let year_month_day = |sep: char| {
        any()
            .filter(|c: &char| c.is_ascii_digit())
            .repeated()
            .at_least(1)
            .at_most(4)
            .collect::<String>()
            .from_str::<i32>()
            .unwrapped()
            .then_ignore(just(sep))
            .then(
                any()
                    .filter(|c: &char| c.is_ascii_digit())
                    .repeated()
                    .at_least(1)
                    .at_most(2)
                    .collect::<String>()
                    .from_str::<u32>()
                    .unwrapped(),
            )
            .then_ignore(just(sep))
            .then(
                any()
                    .filter(|c: &char| c.is_ascii_digit())
                    .repeated()
                    .at_least(1)
                    .at_most(2)
                    .collect::<String>()
                    .from_str::<u32>()
                    .unwrapped(),
            )
            .try_map(|((year, month), day), span| {
                chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or(Rich::custom(
                    span,
                    format!("{year}-{month}-{day} is not a valid date"),
                ))
            })
    };
    year_month_day('.')
        .or(year_month_day('/'))
        .or(year_month_day('-'))
}

// 10/1: October 1st in current year
fn month_day<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    let month_day = |sep: char| {
        any()
            .filter(|c: &char| c.is_ascii_digit())
            .repeated()
            .at_least(1)
            .at_most(2)
            .collect::<String>()
            .from_str::<u32>()
            .unwrapped()
            .then_ignore(just(sep))
            .then(
                any()
                    .filter(|c: &char| c.is_ascii_digit())
                    .repeated()
                    .at_least(1)
                    .at_most(2)
                    .collect::<String>()
                    .from_str::<u32>()
                    .unwrapped(),
            )
            .map_with(|(month, day), e| {
                let state: &mut State = e.state();
                chrono::NaiveDate::from_ymd_opt(state.year, month, day)
            })
            .try_map(|date, span| date.ok_or(Rich::custom(span, "not a valid date")))
    };
    month_day('.').or(month_day('/')).or(month_day('-'))
}

// 2004: start of year
fn four_digits<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    any()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .exactly(4)
        .collect::<String>()
        .from_str::<i32>()
        .unwrapped()
        .try_map(|year, span| {
            chrono::NaiveDate::from_ymd_opt(year, 1, 1).ok_or(Rich::custom(
                span,
                format!("{year}-01-01 is not a valid date"),
            ))
        })
}

// 2004-10: start of month
fn start_of_month_numeric<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    any()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .exactly(4)
        .collect::<String>()
        .from_str::<i32>()
        .unwrapped()
        .then_ignore(one_of("-/."))
        .then(
            any()
                .filter(|c: &char| c.is_ascii_digit())
                .repeated()
                .at_least(1)
                .at_most(2)
                .collect::<String>()
                .from_str::<u32>()
                .unwrapped(),
        )
        .try_map(|(year, month), span| {
            chrono::NaiveDate::from_ymd_opt(year, month, 1).ok_or(Rich::custom(
                span,
                format!("{year}-{month}-01 is not a valid date"),
            ))
        })
}

// 6 digit YYYYMM with valid year and month
fn six_digits<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    any()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .exactly(6)
        .collect::<String>()
        .try_map(|yearmonth, span| {
            let year = yearmonth[0..4]
                .parse::<i32>()
                .map_err(|error| Rich::custom(span, error))?;
            let month = yearmonth[4..6]
                .parse::<u32>()
                .map_err(|error| Rich::custom(span, error))?;
            chrono::NaiveDate::from_ymd_opt(year, month, 1).ok_or(Rich::custom(
                span,
                format!("{year}-{month}-01 is not a valid date"),
            ))
        })
}

// 21: 21st day in current month
fn just_day<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    any()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .exactly(2)
        .collect::<String>()
        .from_str::<u32>()
        .unwrapped()
        .try_map(|day, span| {
            if let Some(date) = today().with_day(day) {
                Ok(date)
            } else {
                Err(Rich::custom(
                    span,
                    format!("{day} day does not exist in the current month"),
                ))
            }
        })
}

// returns today's date
fn today() -> chrono::NaiveDate {
    let current_time = SystemTime::now();
    let datetime: chrono::DateTime<chrono::Local> = current_time.into();
    datetime.date_naive()
}

// oct or october: October 1st in current year
fn month_name<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    let start_of_month = |m: u32| today().with_day(1).unwrap().with_month(m).unwrap();
    choice([
        just("january").to(start_of_month(1)),
        just("jan").to(start_of_month(1)),
        just("february").to(start_of_month(2)),
        just("feb").to(start_of_month(2)),
        just("march").to(start_of_month(3)),
        just("mar").to(start_of_month(3)),
        just("april").to(start_of_month(4)),
        just("apr").to(start_of_month(4)),
        just("may").to(start_of_month(5)),
        just("june").to(start_of_month(6)),
        just("jun").to(start_of_month(6)),
        just("july").to(start_of_month(7)),
        just("jul").to(start_of_month(7)),
        just("august").to(start_of_month(8)),
        just("aug").to(start_of_month(8)),
        just("september").to(start_of_month(9)),
        just("sep").to(start_of_month(9)),
        just("october").to(start_of_month(10)),
        just("oct").to(start_of_month(10)),
        just("november").to(start_of_month(11)),
        just("nov").to(start_of_month(11)),
        just("december").to(start_of_month(12)),
        just("dec").to(start_of_month(12)),
    ])
}

// 8 digit YYYYMMDD with valid year month and day
fn eight_digits<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    any()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .exactly(8)
        .collect::<String>()
        .try_map(|yearmonthday, span| {
            let year = yearmonthday[0..4]
                .parse::<i32>()
                .map_err(|error| Rich::custom(span, error))?;
            let month = yearmonthday[4..6]
                .parse::<u32>()
                .map_err(|error| Rich::custom(span, error))?;
            let day = yearmonthday[6..8]
                .parse::<u32>()
                .map_err(|error| Rich::custom(span, error))?;
            chrono::NaiveDate::from_ymd_opt(year, month, day).ok_or(Rich::custom(
                span,
                format!("{year}-{month}-{day} is not a valid date"),
            ))
        })
}

#[cfg(test)]
mod tests {
    use chumsky::prelude::end;

    use super::*;

    #[test]
    fn start_of_month_numeric() {
        let result = date().then_ignore(end()).parse("2024-03").into_result();
        assert_eq!(
            result,
            Ok(chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()),
        );
    }

    #[test]
    fn start_of_month_numeric_err() {
        let result = date().then_ignore(end()).parse("2024-31").into_result();
        assert!(result.is_err());
    }

    #[test]
    fn four_digits() {
        let result = date().then_ignore(end()).parse("2024").into_result();
        assert_eq!(
            result,
            Ok(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        );
    }

    #[test]
    fn eight_digits() {
        let result = date().then_ignore(end()).parse("20240112").into_result();
        assert_eq!(
            result,
            Ok(chrono::NaiveDate::from_ymd_opt(2024, 1, 12).unwrap()),
        );
    }

    #[test]
    fn eightsix_digits_err() {
        let result = date().then_ignore(end()).parse("20240132").into_result();
        assert!(result.is_err());
    }

    #[test]
    fn six_digits() {
        let result = date().then_ignore(end()).parse("202401").into_result();
        assert_eq!(
            result,
            Ok(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        );
    }

    #[test]
    fn six_digits_err() {
        let result = date().then_ignore(end()).parse("202431").into_result();
        assert!(result.is_err());
    }

    #[test]
    fn year_month_day() {
        let result = date().then_ignore(end()).parse("2018.10.1").into_result();
        assert_eq!(
            result,
            Ok(chrono::NaiveDate::from_ymd_opt(2018, 10, 1).unwrap())
        );
    }

    #[test]
    fn year_month_day_err() {
        let result = date().then_ignore(end()).parse("2023/13/1").into_result();
        assert!(result.is_err());
    }

    #[test]
    fn month_day() {
        let result = date().then_ignore(end()).parse("10/1").into_result();
        assert_eq!(
            result,
            Ok(today().with_month(10).unwrap().with_day(1).unwrap())
        );
    }

    #[test]
    fn month_day_err() {
        let result = date().then_ignore(end()).parse("13/1").into_result();
        assert!(result.is_err());
    }

    #[test]
    fn month_name() {
        let result = date().then_ignore(end()).parse("october").into_result();
        assert_eq!(
            result,
            Ok(today().with_month(10).unwrap().with_day(1).unwrap())
        );
    }

    #[test]
    fn in_three_days() {
        let result = date().then_ignore(end()).parse("in  3 days").into_result();
        assert_eq!(
            result,
            Ok(today().checked_add_days(chrono::Days::new(3)).unwrap())
        );
    }

    #[test]
    fn tomorrow() {
        let result = date().then_ignore(end()).parse("tomorrow").into_result();
        assert_eq!(
            result,
            Ok(today().checked_add_days(chrono::Days::new(1)).unwrap())
        );
    }

    #[test]
    fn test_today() {
        let result = date().then_ignore(end()).parse("today").into_result();
        assert_eq!(result, Ok(today()));
    }

    #[test]
    fn month_ago() {
        let result = date()
            .then_ignore(end())
            .parse("2 months  ago")
            .into_result();
        assert_eq!(
            result,
            Ok(today().checked_sub_months(chrono::Months::new(2)).unwrap())
        );
    }

    #[test]
    fn weeks_ahead() {
        let result = date()
            .then_ignore(end())
            .parse("3 weeks  ahead")
            .into_result();
        assert_eq!(
            result,
            Ok(today().checked_add_days(chrono::Days::new(21)).unwrap())
        );
    }

    #[test]
    fn yesterday() {
        let result = date().then_ignore(end()).parse("yesterday").into_result();
        assert_eq!(
            result,
            Ok(today().checked_sub_days(chrono::Days::new(1)).unwrap())
        );
    }

    #[test]
    fn last_week() {
        let result = date().then_ignore(end()).parse("last week").into_result();
        assert_eq!(
            result,
            Ok(today().checked_sub_days(chrono::Days::new(7)).unwrap())
        );
    }

    #[test]
    fn next_year() {
        let result = date().then_ignore(end()).parse("next year").into_result();
        assert_eq!(
            result,
            Ok(today().checked_add_months(chrono::Months::new(12)).unwrap())
        );
    }

    #[test]
    fn just_day() {
        let result = date().then_ignore(end()).parse("21").into_result();
        assert_eq!(result, Ok(today().with_day(21).unwrap()));
    }

    #[test]
    fn just_day_err() {
        let result = date().then_ignore(end()).parse("32").into_result();
        assert!(result.is_err());
    }
}
