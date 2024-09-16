use chumsky::prelude::*;

use crate::component::whitespace::whitespace;
use crate::state::State;
use crate::utils::end_of_line;

#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub name: String,
}

pub fn tag<'a>() -> impl Parser<'a, &'a str, Tag, extra::Full<Rich<'a, char>, State, ()>> {
    just("tag")
        .ignore_then(whitespace().repeated().at_least(1))
        .ignore_then(
            any()
                .and_is(text::newline().not())
                .and_is(just(";").not())
                .and_is(whitespace().not())
                .repeated()
                .at_least(1)
                .collect::<String>(),
        )
        .then_ignore(end_of_line())
        .map(|tag| Tag {
            name: tag.trim_end().to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_simple() {
        let result = tag().then_ignore(end()).parse("tag test-tag").into_result();
        assert_eq!(
            result,
            Ok(Tag {
                name: String::from("test-tag")
            })
        );
    }

    #[test]
    fn ok_with_comment() {
        let result = tag()
            .then_ignore(end())
            .parse("tag Test ; comment")
            .into_result();
        assert_eq!(
            result,
            Ok(Tag {
                name: String::from("Test")
            })
        );
    }

    #[test]
    fn err_with_space() {
        let result = tag()
            .then_ignore(end())
            .parse("tag Testing things")
            .into_result();
        assert!(result.is_err());
    }

    #[test]
    fn ok_with_trailing() {
        let result = tag().then_ignore(end()).parse("tag 123  ").into_result();
        assert_eq!(
            result,
            Ok(Tag {
                name: String::from("123")
            })
        );
    }

    #[test]
    fn err() {
        let result = tag().then_ignore(end()).parse("t Test").into_result();
        assert!(result.is_err());
    }
}
