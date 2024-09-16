use chumsky::prelude::*;

use crate::state::State;

pub fn date<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveDate, extra::Full<Rich<'a, char>, State, ()>> {
    let digit = any().filter(|c: &char| c.is_ascii_digit());
    let year = digit
        .repeated()
        .exactly(4)
        .collect::<String>()
        .map(|m| m.parse::<i32>().unwrap());
    let month = digit
        .repeated()
        .at_least(1)
        .at_most(2)
        .collect::<String>()
        .map(|m| m.parse::<u32>().unwrap())
        .validate(|s, e, emitter| {
            if !(1..=12).contains(&s) {
                emitter.emit(Rich::custom(
                    e.span(),
                    format!("{s} must be between 1 and 12."),
                ));
            }
            s
        });
    let day = digit
        .repeated()
        .at_least(1)
        .at_most(2)
        .collect::<String>()
        .map(|m| m.parse::<u32>().unwrap())
        .validate(|s, e, emitter| {
            if !(1..=31).contains(&s) {
                emitter.emit(Rich::custom(
                    e.span(),
                    format!("{s} must be between 1 and 31."),
                ));
            }
            s
        });
    let date = |separator: char| {
        year.then_ignore(just(separator))
            .or_not()
            .then(month)
            .then_ignore(just(separator))
            .then(day)
            .map_with(|((year, month), day), e| {
                let state: &mut State = e.state();
                chrono::NaiveDate::from_ymd_opt(year.unwrap_or(state.year), month, day).unwrap()
            })
    };
    date('/').or(date('.')).or(date('-'))
}

#[cfg(test)]
mod tests {
    use chumsky::prelude::end;

    use super::*;

    #[test]
    fn simple() {
        for (input, expected) in [
            (
                "2010-01-31",
                chrono::NaiveDate::from_ymd_opt(2010, 1, 31).unwrap(),
            ),
            (
                "2010-01-31",
                chrono::NaiveDate::from_ymd_opt(2010, 1, 31).unwrap(),
            ),
            (
                "2010/01/31",
                chrono::NaiveDate::from_ymd_opt(2010, 1, 31).unwrap(),
            ),
            (
                "01/31",
                chrono::NaiveDate::from_ymd_opt(2011, 1, 31).unwrap(),
            ),
            (
                "1-31",
                chrono::NaiveDate::from_ymd_opt(2011, 1, 31).unwrap(),
            ),
        ] {
            let result = date()
                .then_ignore(end())
                .parse_with_state(input, &mut State { year: 2011 })
                .into_result();
            assert_eq!(result, Ok(expected), "{input}");
        }
    }
}
