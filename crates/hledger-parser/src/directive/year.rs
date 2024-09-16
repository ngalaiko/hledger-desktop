use chumsky::prelude::*;

use crate::component::whitespace::whitespace;
use crate::state::State;
use crate::utils::end_of_line;

#[derive(Clone, Debug, PartialEq)]
pub struct Year(pub i32);

pub fn year<'a>() -> impl Parser<'a, &'a str, Year, extra::Full<Rich<'a, char>, State, ()>> {
    just("Y")
        .or(just("year").then_ignore(whitespace().repeated().at_least(1)))
        .ignore_then(
            any()
                .filter(|c: &char| c.is_ascii_digit())
                .repeated()
                .at_least(1)
                .collect::<String>()
                .map_with(|p, e| {
                    let state: &mut State = e.state();
                    let year = p.parse::<i32>().unwrap();
                    state.year = year;
                    year
                }),
        )
        .then_ignore(end_of_line())
        .map(Year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_update_state() {
        let mut state = State { year: 1 };
        let result = year()
            .then_ignore(end())
            .parse_with_state("Y2024", &mut state)
            .into_result();
        assert_eq!(result, Ok(Year(2024)));
        assert_eq!(state.year, 2024);
    }

    #[test]
    fn deprecated_form() {
        let result = year()
            .then_ignore(end())
            .parse("year 2024 ; just a comment")
            .into_result();
        assert_eq!(result, Ok(Year(2024)));
    }

    #[test]
    fn ok_with_comment() {
        let result = year()
            .then_ignore(end())
            .parse("Y2024 ; just a comment")
            .into_result();
        assert_eq!(result, Ok(Year(2024)));
    }
}
