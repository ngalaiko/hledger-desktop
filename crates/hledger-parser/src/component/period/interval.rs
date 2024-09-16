use chumsky::prelude::*;

use crate::{component::whitespace::whitespace, state::State};

#[derive(Debug, Clone, PartialEq)]
pub enum Interval {
    // Every N days
    NthDay(u32),
    // Every N Weeks
    NthWeek(u32),
    // Every N quarters
    NthQuarter(u32),
    // Every N months
    NthMonth(u32),
    // Every N years
    NthYear(u32),
    // Weekly on a week day
    Weekday(chrono::Weekday),
}

// TODO:
// every Nth day [of month] (31st day will be adjusted to each month's last day)
// every Nth WEEKDAYNAME [of month]
// Yearly on a custom month and day:
//
// every MM/DD [of year] (month number and day of month number)
// every MONTHNAME DDth [of year] (full or three-letter english month name, case insensitive, and day of month number)
// every DDth MONTHNAME [of year] (equivalent to the above)
pub fn interval<'a>() -> impl Parser<'a, &'a str, Interval, extra::Full<Rich<'a, char>, State, ()>>
{
    let word = choice([
        just("daily").to(Interval::NthDay(1)),
        just("weekly").to(Interval::NthWeek(1)),
        just("biweekly").to(Interval::NthWeek(2)),
        just("fortnightly").to(Interval::NthWeek(2)),
        just("monthly").to(Interval::NthMonth(1)),
        just("bimonthly").to(Interval::NthMonth(2)),
        just("quarterly").to(Interval::NthQuarter(1)),
        just("yearly").to(Interval::NthQuarter(1)),
    ]);

    word.or(every()).or(day_of_week())
}

fn day_of_week<'a>() -> impl Parser<'a, &'a str, Interval, extra::Full<Rich<'a, char>, State, ()>> {
    let monday = just("monday")
        .ignored()
        .or(just("mon").ignored())
        .or(just("1st")
            .then(whitespace().repeated().at_least(1))
            .then(just("day"))
            .then(whitespace().repeated().at_least(1))
            .then(just("of"))
            .then(whitespace().repeated().at_least(1))
            .then(just("week"))
            .ignored())
        .map(|()| Interval::Weekday(chrono::Weekday::Mon));
    let tuesday = just("tuesday")
        .ignored()
        .or(just("tue").ignored())
        .or(just("2nd")
            .then(whitespace().repeated().at_least(1))
            .then(just("day"))
            .then(whitespace().repeated().at_least(1))
            .then(just("of"))
            .then(whitespace().repeated().at_least(1))
            .then(just("week"))
            .ignored())
        .map(|()| Interval::Weekday(chrono::Weekday::Tue));
    let wednesday = just("wednesday")
        .ignored()
        .or(just("wed").ignored())
        .or(just("3rd")
            .then(whitespace().repeated().at_least(1))
            .then(just("day"))
            .then(whitespace().repeated().at_least(1))
            .then(just("of"))
            .then(whitespace().repeated().at_least(1))
            .then(just("week"))
            .ignored())
        .map(|()| Interval::Weekday(chrono::Weekday::Wed));
    let thursday = just("thursday")
        .ignored()
        .or(just("thu").ignored())
        .or(just("3rd")
            .then(whitespace().repeated().at_least(1))
            .then(just("day"))
            .then(whitespace().repeated().at_least(1))
            .then(just("of"))
            .then(whitespace().repeated().at_least(1))
            .then(just("week"))
            .ignored())
        .map(|()| Interval::Weekday(chrono::Weekday::Thu));
    let friday = just("friday")
        .ignored()
        .or(just("fri").ignored())
        .or(just("3rd")
            .then(whitespace().repeated().at_least(1))
            .then(just("day"))
            .then(whitespace().repeated().at_least(1))
            .then(just("of"))
            .then(whitespace().repeated().at_least(1))
            .then(just("week"))
            .ignored())
        .map(|()| Interval::Weekday(chrono::Weekday::Fri));
    let saturday = just("saturday")
        .ignored()
        .or(just("sat").ignored())
        .or(just("3rd")
            .then(whitespace().repeated().at_least(1))
            .then(just("day"))
            .then(whitespace().repeated().at_least(1))
            .then(just("of"))
            .then(whitespace().repeated().at_least(1))
            .then(just("week"))
            .ignored())
        .map(|()| Interval::Weekday(chrono::Weekday::Sat));
    let sunday = just("sunday")
        .ignored()
        .or(just("sun").ignored())
        .or(just("3rd")
            .then(whitespace().repeated().at_least(1))
            .then(just("day"))
            .then(whitespace().repeated().at_least(1))
            .then(just("of"))
            .then(whitespace().repeated().at_least(1))
            .then(just("week"))
            .ignored())
        .map(|()| Interval::Weekday(chrono::Weekday::Sun));
    just("every")
        .then(whitespace().repeated().at_least(1))
        .ignore_then(
            monday
                .or(tuesday)
                .or(wednesday)
                .or(thursday)
                .or(friday)
                .or(saturday)
                .or(sunday),
        )
}

fn every<'a>() -> impl Parser<'a, &'a str, Interval, extra::Full<Rich<'a, char>, State, ()>> {
    let every = just("every")
        .then(whitespace().repeated().at_least(1))
        .ignore_then(choice([
            just("day").to(Interval::NthDay(1)),
            just("week").to(Interval::NthWeek(1)),
            just("month").to(Interval::NthMonth(1)),
            just("quarter").to(Interval::NthQuarter(1)),
            just("year").to(Interval::NthYear(1)),
        ]));
    let every_n_days = just("every")
        .then(whitespace().repeated().at_least(1))
        .ignore_then(text::int(10).from_str::<u32>().unwrapped())
        .then_ignore(whitespace().repeated().at_least(1))
        .then_ignore(just("days"))
        .map(Interval::NthDay);
    let every_n_weeks = just("every")
        .then(whitespace().repeated().at_least(1))
        .ignore_then(text::int(10).from_str::<u32>().unwrapped())
        .then_ignore(whitespace().repeated().at_least(1))
        .then_ignore(just("weeks"))
        .map(Interval::NthWeek);
    let every_n_months = just("every")
        .then(whitespace().repeated().at_least(1))
        .ignore_then(text::int(10).from_str::<u32>().unwrapped())
        .then_ignore(whitespace().repeated().at_least(1))
        .then_ignore(just("months"))
        .map(Interval::NthMonth);
    let every_n_quarterd = just("every")
        .then(whitespace().repeated().at_least(1))
        .ignore_then(text::int(10).from_str::<u32>().unwrapped())
        .then_ignore(whitespace().repeated().at_least(1))
        .then_ignore(just("quarters"))
        .map(Interval::NthQuarter);
    let every_n_years = just("every")
        .then(whitespace().repeated().at_least(1))
        .ignore_then(text::int(10).from_str::<u32>().unwrapped())
        .then_ignore(whitespace().repeated().at_least(1))
        .then_ignore(just("years"))
        .map(Interval::NthYear);
    let every_n = every_n_days
        .or(every_n_weeks)
        .or(every_n_months)
        .or(every_n_quarterd)
        .or(every_n_years);
    every.or(every_n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_day() {
        let result = interval()
            .then_ignore(end())
            .parse("every  day")
            .into_result();
        assert_eq!(result, Ok(Interval::NthDay(1)));
    }

    #[test]
    fn every_n_day() {
        let result = interval()
            .then_ignore(end())
            .parse("every 3 days")
            .into_result();
        assert_eq!(result, Ok(Interval::NthDay(3)));
    }

    #[test]
    fn every_week() {
        let result = interval()
            .then_ignore(end())
            .parse("every week")
            .into_result();
        assert_eq!(result, Ok(Interval::NthWeek(1)));
    }

    #[test]
    fn every_n_week() {
        let result = interval()
            .then_ignore(end())
            .parse("every 4 weeks")
            .into_result();
        assert_eq!(result, Ok(Interval::NthWeek(4)));
    }

    #[test]
    fn every_month() {
        let result = interval()
            .then_ignore(end())
            .parse("every month")
            .into_result();
        assert_eq!(result, Ok(Interval::NthMonth(1)));
    }

    #[test]
    fn every_n_month() {
        let result = interval()
            .then_ignore(end())
            .parse("every 2 months")
            .into_result();
        assert_eq!(result, Ok(Interval::NthMonth(2)));
    }

    #[test]
    fn every_quarter() {
        let result = interval()
            .then_ignore(end())
            .parse("every quarter")
            .into_result();
        assert_eq!(result, Ok(Interval::NthQuarter(1)));
    }

    #[test]
    fn every_n_quarter() {
        let result = interval()
            .then_ignore(end())
            .parse("every 2 quarters")
            .into_result();
        assert_eq!(result, Ok(Interval::NthQuarter(2)));
    }

    #[test]
    fn every_year() {
        let result = interval()
            .then_ignore(end())
            .parse("every year")
            .into_result();
        assert_eq!(result, Ok(Interval::NthYear(1)));
    }

    #[test]
    fn every_n_years() {
        let result = interval()
            .then_ignore(end())
            .parse("every 10 years")
            .into_result();
        assert_eq!(result, Ok(Interval::NthYear(10)));
    }

    #[test]
    fn every_weekday() {
        let result = interval()
            .then_ignore(end())
            .parse("every tue")
            .into_result();
        assert_eq!(result, Ok(Interval::Weekday(chrono::Weekday::Tue)));
    }
}
