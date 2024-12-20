use chumsky::prelude::*;

use crate::state::State;

#[derive(Clone, Debug, PartialEq)]
pub enum Format {
    Journal,
    Timeclock,
    Timedot,
    Csv,
    Ssv,
    Tsv,
    Rules,
}

pub fn format<'a>() -> impl Parser<'a, &'a str, Format, extra::Full<Rich<'a, char>, State, ()>> {
    let journal = just("journal").map(|_| Format::Journal);
    let timeclock = just("timeclock").map(|_| Format::Timeclock);
    let timedot = just("timedot").map(|_| Format::Timedot);
    let comma_sv = just("csv").map(|_| Format::Csv);
    let semicolon_sv = just("ssv").map(|_| Format::Ssv);
    let tab_sv = just("tsv").map(|_| Format::Tsv);
    let rules = just("rules").map(|_| Format::Rules);
    choice((
        journal,
        timeclock,
        timedot,
        comma_sv,
        semicolon_sv,
        tab_sv,
        rules,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_journal() {
        let result = format().then_ignore(end()).parse("journal").into_result();
        assert_eq!(result, Ok(Format::Journal));
    }

    #[test]
    fn ok_timeclock() {
        let result = format().then_ignore(end()).parse("timeclock").into_result();
        assert_eq!(result, Ok(Format::Timeclock));
    }

    #[test]
    fn ok_timedot() {
        let result = format().then_ignore(end()).parse("timedot").into_result();
        assert_eq!(result, Ok(Format::Timedot));
    }

    #[test]
    fn ok_csv() {
        let result = format().then_ignore(end()).parse("csv").into_result();
        assert_eq!(result, Ok(Format::Csv));
    }

    #[test]
    fn ok_ssv() {
        let result = format().then_ignore(end()).parse("ssv").into_result();
        assert_eq!(result, Ok(Format::Ssv));
    }

    #[test]
    fn ok_tsv() {
        let result = format().then_ignore(end()).parse("tsv").into_result();
        assert_eq!(result, Ok(Format::Tsv));
    }

    #[test]
    fn ok_rules() {
        let result = format().then_ignore(end()).parse("rules").into_result();
        assert_eq!(result, Ok(Format::Rules));
    }

    #[test]
    fn err() {
        let result = format().then_ignore(end()).parse("err").into_result();
        assert!(result.is_err());
    }
}
