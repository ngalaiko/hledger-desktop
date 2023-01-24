use nom::{
    bytes::complete::take_until,
    character::{complete::char, complete::space0, is_digit, is_newline, is_space},
    sequence::{delimited, preceded, terminated},
    IResult,
};

pub fn is_char_newline(char: char) -> bool {
    is_newline(char as u8)
}

pub fn is_char_space(char: char) -> bool {
    is_space(char as u8)
}

pub fn is_char_digit(char: char) -> bool {
    is_digit(char as u8)
}

pub fn is_char_minus(char: char) -> bool {
    char == '-'
}

pub fn is_char_alphanumeric(char: char) -> bool {
    char.is_alphanumeric()
}

pub fn in_quotes(input: &str) -> IResult<&str, &str> {
    delimited(
        terminated(char('"'), space0),
        take_until("\""),
        preceded(space0, char('"')),
    )(input)
}

pub fn split_on_space_before_char(input: &str, char: char) -> (&str, &str) {
    let char_pos = input.find(char);
    let space_pos = match char_pos {
        Some(pos) => input[..pos].rfind(' '),
        None => return (input, ""),
    };

    match space_pos {
        Some(pos) => (input[..pos].into(), input[pos + 1..].into()),
        None => (input, ""),
    }
}

#[cfg(test)]
mod tests {
    use crate::hledger::utils::split_on_space_before_char;

    #[test]
    fn test_split_on_space_before_char() {
        assert_eq!(
            split_on_space_before_char("before before source:salary", ':'),
            ("before before", "source:salary")
        );
        assert_eq!(
            split_on_space_before_char("преди преди източник:заплата", ':'),
            ("преди преди", "източник:заплата")
        );
        assert_eq!(split_on_space_before_char("", ':'), ("", ""));
    }
}
