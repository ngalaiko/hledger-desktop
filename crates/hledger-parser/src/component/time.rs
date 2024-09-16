use chumsky::prelude::*;

use crate::state::State;

pub fn time<'a>(
) -> impl Parser<'a, &'a str, chrono::NaiveTime, extra::Full<Rich<'a, char>, State, ()>> {
    let digit = any().filter(move |c: &char| c.is_ascii_digit());
    let hour = digit
        .repeated()
        .exactly(2)
        .collect::<String>()
        .from_str::<u32>()
        .unwrapped();

    let minute = digit
        .repeated()
        .exactly(2)
        .collect::<String>()
        .from_str::<u32>()
        .unwrapped();
    let second = digit
        .repeated()
        .exactly(2)
        .collect::<String>()
        .from_str::<u32>()
        .unwrapped();

    hour.then_ignore(just(":"))
        .then(minute)
        .then_ignore(just(":"))
        .then(second)
        .try_map(|((hours, minutes), seconds), span| {
            chrono::NaiveTime::from_hms_opt(hours, minutes, seconds)
                .ok_or(Rich::custom(span, "invalid time"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let result = time().then_ignore(end()).parse("00:00:00").into_result();
        assert_eq!(
            result,
            Ok(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        );
    }
    #[test]
    fn error() {
        let result = time().then_ignore(end()).parse("25:00:00").into_result();
        assert!(result.is_err());
    }
}
