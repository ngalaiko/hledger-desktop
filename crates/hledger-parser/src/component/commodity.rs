use chumsky::prelude::*;

use crate::state::State;

pub fn commodity<'a>() -> impl Parser<'a, &'a str, String, extra::Full<Rich<'a, char>, State, ()>> {
    let letter = any().filter(|c: &char| c.is_alphabetic());
    let symbol = one_of("$¢€£ƒ₣₧₱₨₹₽₺¥");

    let symbol = symbol.repeated().exactly(1).collect();
    let simple = letter.repeated().collect();
    let quoted = any()
        .and_is(just("\"").not())
        .repeated()
        .collect::<String>()
        .padded_by(just("\""));

    symbol.or(quoted).or(simple)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn symbol() {
        let result = commodity().then_ignore(end()).parse("$").into_result();
        assert_eq!(result, Ok(String::from("$")));
    }

    #[test]
    pub fn simple() {
        let result = commodity().then_ignore(end()).parse("USD").into_result();
        assert_eq!(result, Ok(String::from("USD")));
    }

    #[test]
    pub fn quoted() {
        let result = commodity()
            .then_ignore(end())
            .parse("\"green apples\"")
            .into_result();
        assert_eq!(result, Ok(String::from("green apples")));
    }

    #[test]
    pub fn complex_quoted() {
        let result = commodity()
            .then_ignore(end())
            .parse("\"Hello 1 - I am a very complex commodity 😎\"")
            .into_result();
        assert_eq!(
            result,
            Ok(String::from("Hello 1 - I am a very complex commodity 😎"))
        );
    }

    #[test]
    pub fn error() {
        let result = commodity().then_ignore(end()).parse("123").into_errors();
        assert!(!result.is_empty());
    }
}
