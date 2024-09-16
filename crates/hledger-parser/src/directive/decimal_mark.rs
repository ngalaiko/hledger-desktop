use chumsky::prelude::*;

use crate::component::whitespace::whitespace;
use crate::state::State;
use crate::utils::end_of_line;

#[derive(Clone, Debug, PartialEq)]
pub struct DecimalMark(pub char);

pub fn decimal_mark<'a>(
) -> impl Parser<'a, &'a str, DecimalMark, extra::Full<Rich<'a, char>, State, ()>> {
    just("decimal-mark")
        .ignore_then(whitespace().repeated().at_least(1))
        .ignore_then(one_of(".,"))
        .then_ignore(end_of_line())
        .map(DecimalMark)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_trailing() {
        let result = decimal_mark()
            .then_ignore(end())
            .parse("decimal-mark , ")
            .into_result();
        assert_eq!(result, Ok(DecimalMark(',')));
    }

    #[test]
    fn ok_comma() {
        let result = decimal_mark()
            .then_ignore(end())
            .parse("decimal-mark ,")
            .into_result();
        assert_eq!(result, Ok(DecimalMark(',')));
    }

    #[test]
    fn ok_dot() {
        let result = decimal_mark()
            .then_ignore(end())
            .parse("decimal-mark .")
            .into_result();
        assert_eq!(result, Ok(DecimalMark('.')));
    }

    #[test]
    fn ok_comment() {
        let result = decimal_mark()
            .then_ignore(end())
            .parse("decimal-mark .  ; test")
            .into_result();
        assert_eq!(result, Ok(DecimalMark('.')));
    }

    #[test]
    fn err_format() {
        let result = decimal_mark()
            .then_ignore(end())
            .parse("decimal-mark ")
            .into_result();
        assert!(result.is_err());
    }
}
