use chumsky::prelude::*;

use crate::component::whitespace::whitespace;
use crate::state::State;

#[derive(Clone, Debug, PartialEq)]
pub struct Comment(String);

pub fn line<'a>() -> impl Parser<'a, &'a str, Comment, extra::Full<Rich<'a, char>, State, ()>> {
    just("#")
        .ignore_then(
            any()
                .and_is(text::newline().not())
                .repeated()
                .collect::<String>(),
        )
        .map(Comment)
}

pub fn block<'a>() -> impl Parser<'a, &'a str, Comment, extra::Full<Rich<'a, char>, State, ()>> {
    any()
        .and_is(text::newline().not())
        .and_is(just("end comment\n").not())
        .repeated()
        .collect::<String>()
        .separated_by(text::newline())
        .collect::<Vec<_>>()
        .delimited_by(just("comment\n"), just("end comment\n"))
        .map(|lines| Comment(lines.join("\n").trim().to_string()))
}

pub fn inline<'a>() -> impl Parser<'a, &'a str, Comment, extra::Full<Rich<'a, char>, State, ()>> {
    let comment = just(";").ignore_then(
        any()
            .and_is(text::newline().not())
            .repeated()
            .collect::<String>(),
    );
    let prefixed_comment =
        text::newline().ignore_then(whitespace().repeated().at_least(1).ignore_then(comment));
    comment
        .then(prefixed_comment.repeated().collect::<Vec<_>>())
        .map(|(first, rest)| {
            Comment(
                std::iter::once(first)
                    .chain(rest)
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_line() {
        let result = line().then_ignore(end()).parse("# a comment").into_result();
        assert_eq!(result, Ok(Comment(" a comment".to_string())));
    }

    #[test]
    fn ok_inline() {
        let result = inline()
            .then_ignore(end())
            .parse("; a comment")
            .into_result();
        assert_eq!(result, Ok(Comment(" a comment".to_string())));
    }

    #[test]
    fn ok_block() {
        let result = block()
            .then_ignore(end())
            .parse("comment\nmultiline\ncomment block\nend comment\n")
            .into_result();
        assert_eq!(result, Ok(Comment("multiline\ncomment block".to_string())));
    }

    #[test]
    fn inline_multiline() {
        let result = inline()
            .then_ignore(end())
            .parse("; a comment\n ; continuation")
            .into_result();
        assert_eq!(result, Ok(Comment(" a comment\n continuation".to_string())));
    }

    #[test]
    fn err() {
        let result = inline()
            .then_ignore(end())
            .parse("not a comment")
            .into_result();
        assert!(result.is_err());
    }
}
