//! Parser for Hledger journals.
//! See [hledger documentation](https://hledger.org/hledger.html)
//! for journal format description.

mod component;

mod directive;
mod state;
mod utils;

use chumsky::error::RichReason;
use chumsky::prelude::*;

use crate::directive::directives;
use crate::state::State;

pub use crate::component::amount::Amount;
pub use crate::component::interval::Interval;
pub use crate::component::period::Period;
pub use crate::component::price::AmountPrice;
pub use crate::component::query::{Condition, Term};
pub use crate::component::status::Status;
pub use crate::directive::{
    Account, Assertion, AutoPosting, AutosPostingRule, Commodity, DecimalMark, Directive, Format,
    Include, Payee, PeriodicTransaction, Posting, Price, Tag, Transaction, Year,
};

use crate::component::query::query;

/// Parses the given content into a hledger query.
///
/// # Errors
///
/// Will return a list of parsing errors if input is not a query.
pub fn parse_query<I: AsRef<str>>(contents: I) -> Result<Vec<Term>, Vec<ParseError>> {
    query()
        .then_ignore(end())
        .parse_with_state(contents.as_ref(), &mut State::default())
        .into_result()
        .map_err(|errors| errors.into_iter().map(ParseError::from).collect())
}

/// Parses the given content into a list of Hledger journal directives.
///
/// # Errors
///
/// Will return a list of parsing errors if input is not a valid hledger journal.
pub fn parse<I: AsRef<str>>(contents: I) -> Result<Vec<Directive>, Vec<ParseError>> {
    directives()
        .then_ignore(end())
        .parse_with_state(contents.as_ref(), &mut State::default())
        .into_result()
        .map_err(|errors| errors.into_iter().map(ParseError::from).collect())
}

/// Error type representing failures during parsing.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// The span of text where the error occurred.
    pub span: std::ops::Range<usize>,
    /// A human-readable description of the error.
    pub message: String,
}

impl<'a, T: std::fmt::Debug> From<Rich<'a, T>> for ParseError {
    fn from(rich_error: Rich<'a, T>) -> Self {
        let span = rich_error.span().start..rich_error.span().end;

        let message = match rich_error.into_reason() {
            RichReason::Custom(msg) => msg,
            RichReason::ExpectedFound { expected, found } => {
                let expected_items: Vec<_> = expected
                    .into_iter()
                    .map(|pattern| format!("{pattern:?}"))
                    .collect();

                format!(
                    "Expected {}{}",
                    if expected_items.is_empty() {
                        String::from("something else")
                    } else {
                        format!("one of: [{}]", expected_items.join(", "))
                    },
                    if let Some(found) = found {
                        format!(", found: {found:?}")
                    } else {
                        String::new()
                    }
                )
            }
            RichReason::Many(reasons) => reasons
                .into_iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("; "),
        };

        ParseError { span, message }
    }
}
