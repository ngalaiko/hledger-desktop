pub use hledger_parser::{ParseError, Posting, Transaction};

type PostingFilter = Box<dyn for<'a> Fn(&'a Posting) -> bool>;

pub struct Query {
    filter: transaction::Filter,
    posting_filter: PostingFilter,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            filter: Box::new(|_| true),
            posting_filter: Box::new(|_| true),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to parse")]
    Parse(Vec<ParseError>),
    #[error(transparent)]
    Regex(regex::Error),
}

impl Query {
    #[allow(clippy::missing_errors_doc)]
    pub fn parse(query: &str) -> Result<Query, Error> {
        let terms = hledger_parser::parse_query(query).map_err(Error::Parse)?;
        let transaction_filters = terms.iter().map(to_filter).collect::<Result<Vec<_>, _>>()?;
        let posting_filters = terms
            .iter()
            .map(to_posting_filter)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            filter: Box::new(move |tx| transaction_filters.iter().all(|f| f(tx))),
            posting_filter: Box::new(move |posting| posting_filters.iter().all(|f| f(posting))),
        })
    }

    #[must_use]
    pub fn filter(&self, tx: &Transaction) -> Option<Transaction> {
        if !(self.filter)(tx) {
            return None;
        }

        let postings = tx
            .postings
            .iter()
            .filter(|posting| (self.posting_filter)(posting))
            .cloned()
            .collect::<Vec<_>>();
        if postings.is_empty() {
            return None;
        }

        Some(Transaction {
            postings,
            ..tx.clone()
        })
    }
}

fn to_filter(term: &hledger_parser::Term) -> Result<transaction::Filter, Error> {
    let filter = match &term.condition {
        hledger_parser::Condition::Account(_)
        | hledger_parser::Condition::Currency(_)
        | hledger_parser::Condition::Amount(_) => Ok(transaction::always_true_filter()),
        hledger_parser::Condition::Code(query) => transaction::code_filter(query),
        hledger_parser::Condition::Description(query) => transaction::description_filter(query),
        hledger_parser::Condition::Note(query) => transaction::note_filter(query),
        hledger_parser::Condition::Payee(query) => transaction::payee_filter(query),
        hledger_parser::Condition::Date(period) => Ok(transaction::date_filter(period)),
        hledger_parser::Condition::Status(status) => {
            Ok(transaction::status_filter(status.as_ref()))
        }
    }?;
    if term.is_not {
        Ok(transaction::not_filter(filter))
    } else {
        Ok(filter)
    }
}

#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::match_single_binding)]
fn to_posting_filter(term: &hledger_parser::Term) -> Result<PostingFilter, Error> {
    let is_not = term.is_not;
    match &term.condition {
        hledger_parser::Condition::Account(r) => {
            let r = regex::RegexBuilder::new(r)
                .case_insensitive(true)
                .build()
                .map_err(Error::Regex)?;
            Ok(Box::new(move |posting| {
                let is_match = r.is_match(&posting.account_name.to_string());
                (is_match && !is_not) || (!is_match && is_not)
            }))
        }
        _ => Ok(Box::new(|_| true)),
    }
}

mod transaction {
    use hledger_parser::{Period, Status};

    use super::{Error, Transaction};

    pub type Filter = Box<dyn for<'a> Fn(&'a Transaction) -> bool>;

    pub fn not_filter(filter: Filter) -> Filter {
        Box::new(move |tx| !filter(tx))
    }

    pub fn description_filter(query: &str) -> Result<Filter, Error> {
        let r = regex::RegexBuilder::new(query)
            .case_insensitive(true)
            .build()
            .map_err(Error::Regex)?;
        Ok(Box::new(move |tx| {
            if let Some(note) = &tx.note {
                r.is_match(&format!("{} | {note}", tx.payee))
            } else {
                r.is_match(&tx.payee)
            }
        }))
    }

    pub fn date_filter(date: &Period) -> Filter {
        let begin = date.begin;
        let end = date.end;
        Box::new(move |tx| {
            if let Some(begin) = begin {
                if tx.date < begin {
                    return false;
                }
            }
            if let Some(end) = end {
                if tx.date >= end {
                    return false;
                }
            }
            true
        })
    }

    pub fn status_filter(status: Option<&Status>) -> Filter {
        let status = status.cloned();
        Box::new(move |tx| tx.status == status)
    }

    pub fn payee_filter(query: &str) -> Result<Filter, Error> {
        let r = regex::RegexBuilder::new(query)
            .case_insensitive(true)
            .build()
            .map_err(Error::Regex)?;
        Ok(Box::new(move |tx| r.is_match(&tx.payee)))
    }

    pub fn note_filter(query: &str) -> Result<Filter, Error> {
        let r = regex::RegexBuilder::new(query)
            .case_insensitive(true)
            .build()
            .map_err(Error::Regex)?;
        Ok(Box::new(move |tx| {
            tx.note.as_ref().map_or(true, |note| r.is_match(note))
        }))
    }

    pub fn always_true_filter() -> Filter {
        Box::new(|_| true)
    }

    pub fn code_filter(query: &str) -> Result<Filter, Error> {
        let r = regex::RegexBuilder::new(query)
            .case_insensitive(true)
            .build()
            .map_err(Error::Regex)?;
        Ok(Box::new(move |tx| {
            tx.code.as_ref().map_or(true, |note| r.is_match(note))
        }))
    }
}
