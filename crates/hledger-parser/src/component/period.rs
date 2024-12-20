#![allow(dead_code)]

pub mod interval;

use std::time::SystemTime;

use chrono::Datelike;
use chumsky::prelude::*;

use crate::component::date::smart::date;
use crate::component::period::interval::{interval, Interval};
use crate::component::whitespace::whitespace;
use crate::state::State;

#[derive(Debug, Clone, PartialEq)]
pub struct Period {
    pub interval: Option<Interval>,
    pub begin: Option<chrono::NaiveDate>,
    pub end: Option<chrono::NaiveDate>,
}

pub fn period<'a>() -> impl Parser<'a, &'a str, Period, extra::Full<Rich<'a, char>, State, ()>> {
    let interval_begin_end = interval()
        .then_ignore(
            whitespace()
                .repeated()
                .at_least(1)
                .ignore_then(just("in"))
                .ignore_then(whitespace().repeated().at_least(1))
                .or(whitespace().repeated().at_least(1)),
        )
        .then(
            quarter()
                .or(year_quarter())
                .or(begin_end())
                .or(just_end())
                .or(begin())
                .or(year_month_day())
                .or(year_month())
                .or(year()),
        )
        .map(|(interval, (begin, end))| Period {
            interval: Some(interval),
            begin,
            end,
        });

    let interval = interval().map(|interval| Period {
        interval: Some(interval),
        begin: None,
        end: None,
    });

    let begin_end = choice((
        quarter(),
        year_quarter(),
        begin_end(),
        just_end(),
        begin(),
        year_month_day(),
        year_month(),
        year(),
    ))
    .map(|(begin, end)| Period {
        interval: None,
        begin,
        end,
    });

    choice((interval_begin_end, interval, begin_end))
}

// returns today's date
fn today() -> chrono::NaiveDate {
    let current_time = SystemTime::now();
    let datetime: chrono::DateTime<chrono::Local> = current_time.into();
    datetime.date_naive()
}

// 2009Q1
fn year_quarter<'a>() -> impl Parser<
    'a,
    &'a str,
    (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
    extra::Full<Rich<'a, char>, State, ()>,
> {
    any()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .exactly(4)
        .collect::<String>()
        .from_str::<i32>()
        .unwrapped()
        .then(
            one_of("qQ")
                .ignore_then(one_of("1234"))
                .map(|s: char| s.to_string())
                .from_str::<u32>()
                .unwrapped(),
        )
        .map(|(year, q)| {
            (
                chrono::NaiveDate::from_ymd_opt(year, (q - 1) * 3 + 1, 1),
                chrono::NaiveDate::from_ymd_opt(year, (q - 1) * 3 + 4, 1),
            )
        })
}

// q1
fn quarter<'a>() -> impl Parser<
    'a,
    &'a str,
    (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
    extra::Full<Rich<'a, char>, State, ()>,
> {
    one_of("qQ")
        .ignore_then(one_of("1234"))
        .map(|s: char| s.to_string())
        .from_str::<u32>()
        .unwrapped()
        .map(|q| {
            (
                today().with_month((q - 1) * 3 + 1).unwrap().with_day(1),
                today().with_month((q - 1) * 3 + 4).unwrap().with_day(1),
            )
        })
}

fn begin_end<'a>() -> impl Parser<
    'a,
    &'a str,
    (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
    extra::Full<Rich<'a, char>, State, ()>,
> {
    just("from")
        .or(just("since"))
        .ignore_then(whitespace().repeated().at_least(1))
        .or_not()
        .ignore_then(date())
        .then_ignore(
            whitespace()
                .repeated()
                .then(just("to").or(just("..")).or(just("-")))
                .or_not(),
        )
        .then_ignore(whitespace().repeated())
        .then(date())
        .map(|(begin, end)| (Some(begin), Some(end)))
}

// to 2009
fn just_end<'a>() -> impl Parser<
    'a,
    &'a str,
    (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
    extra::Full<Rich<'a, char>, State, ()>,
> {
    just("to")
        .then(whitespace().repeated())
        .ignore_then(date())
        .map(|end| (None, Some(end)))
}

// since 2009
fn begin<'a>() -> impl Parser<
    'a,
    &'a str,
    (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
    extra::Full<Rich<'a, char>, State, ()>,
> {
    just("from")
        .or(just("since"))
        .ignore_then(whitespace().repeated().at_least(1))
        .ignore_then(date())
        .map(|begin| (Some(begin), None))
}

// 2009
fn year<'a>() -> impl Parser<
    'a,
    &'a str,
    (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
    extra::Full<Rich<'a, char>, State, ()>,
> {
    any()
        .filter(|c: &char| c.is_ascii_digit())
        .repeated()
        .exactly(4)
        .collect::<String>()
        .from_str::<i32>()
        .unwrapped()
        .map(|year| {
            (
                chrono::NaiveDate::from_ymd_opt(year, 1, 1),
                chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1),
            )
        })
}

// 2009/1
fn year_month<'a>() -> impl Parser<
    'a,
    &'a str,
    (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
    extra::Full<Rich<'a, char>, State, ()>,
> {
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
        .map(|(year, month)| {
            (
                chrono::NaiveDate::from_ymd_opt(year, month, 1),
                chrono::NaiveDate::from_ymd_opt(year, month + 1, 1),
            )
        })
}

// 2024/10/1: exact date
fn year_month_day<'a>() -> impl Parser<
    'a,
    &'a str,
    (Option<chrono::NaiveDate>, Option<chrono::NaiveDate>),
    extra::Full<Rich<'a, char>, State, ()>,
> {
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
            .map(|((year, month), day)| {
                (
                    chrono::NaiveDate::from_ymd_opt(year, month, day),
                    chrono::NaiveDate::from_ymd_opt(year, month, day + 1),
                )
            })
    };
    year_month_day('.')
        .or(year_month_day('/'))
        .or(year_month_day('-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_to() {
        let result = period()
            .then_ignore(end())
            .parse("from 2009/1/1 to 2009/4/1")
            .into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: None,
                begin: Some(chrono::NaiveDate::from_ymd_opt(2009, 1, 1).unwrap()),
                end: Some(chrono::NaiveDate::from_ymd_opt(2009, 4, 1).unwrap()),
            })
        );
    }

    #[test]
    fn dots() {
        let result = period()
            .then_ignore(end())
            .parse("2009/1/1..2009/4/1")
            .into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: None,
                begin: Some(chrono::NaiveDate::from_ymd_opt(2009, 1, 1).unwrap()),
                end: Some(chrono::NaiveDate::from_ymd_opt(2009, 4, 1).unwrap()),
            })
        );
    }

    #[test]
    fn only_begin() {
        let result = period()
            .then_ignore(end())
            .parse("since 2009/1")
            .into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: None,
                begin: Some(chrono::NaiveDate::from_ymd_opt(2009, 1, 1).unwrap()),
                end: None,
            })
        );
    }

    #[test]
    fn only_end() {
        let result = period().then_ignore(end()).parse("to 2009").into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: None,
                begin: None,
                end: Some(chrono::NaiveDate::from_ymd_opt(2009, 1, 1).unwrap()),
            })
        );
    }

    #[test]
    fn year() {
        let result = period().then_ignore(end()).parse("2009").into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: None,
                begin: Some(chrono::NaiveDate::from_ymd_opt(2009, 1, 1).unwrap()),
                end: Some(chrono::NaiveDate::from_ymd_opt(2010, 1, 1).unwrap()),
            })
        );
    }

    #[test]
    fn month() {
        let result = period().then_ignore(end()).parse("2009/1").into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: None,
                begin: Some(chrono::NaiveDate::from_ymd_opt(2009, 1, 1).unwrap()),
                end: Some(chrono::NaiveDate::from_ymd_opt(2009, 2, 1).unwrap()),
            })
        );
    }

    #[test]
    fn day() {
        let result = period().then_ignore(end()).parse("2009/1/1").into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: None,
                begin: Some(chrono::NaiveDate::from_ymd_opt(2009, 1, 1).unwrap()),
                end: Some(chrono::NaiveDate::from_ymd_opt(2009, 1, 2).unwrap()),
            })
        );
    }

    #[test]
    fn quarter() {
        let result = period().then_ignore(end()).parse("q3").into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: None,
                begin: today().with_month(7).unwrap().with_day(1),
                end: today().with_month(10).unwrap().with_day(1),
            })
        );
    }

    #[test]
    fn year_quarter() {
        let result = period().then_ignore(end()).parse("2009Q3").into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: None,
                begin: chrono::NaiveDate::from_ymd_opt(2009, 7, 1),
                end: chrono::NaiveDate::from_ymd_opt(2009, 10, 1),
            })
        );
    }

    #[test]
    fn with_in_interval() {
        let result = period()
            .then_ignore(end())
            .parse("every 2 weeks in 2008")
            .into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: Some(Interval::NthWeek(2)),
                begin: chrono::NaiveDate::from_ymd_opt(2008, 1, 1),
                end: chrono::NaiveDate::from_ymd_opt(2009, 1, 1),
            })
        );
    }

    #[test]
    fn with_interval() {
        let result = period()
            .then_ignore(end())
            .parse("weekly from 2009/1/1 to 2009/4/1")
            .into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: Some(Interval::NthWeek(1)),
                begin: chrono::NaiveDate::from_ymd_opt(2009, 1, 1),
                end: chrono::NaiveDate::from_ymd_opt(2009, 4, 1),
            })
        );
    }

    #[test]
    fn just_interval() {
        let result = period().then_ignore(end()).parse("monthly").into_result();
        assert_eq!(
            result,
            Ok(Period {
                interval: Some(Interval::NthMonth(1)),
                begin: None,
                end: None,
            })
        );
    }
}
