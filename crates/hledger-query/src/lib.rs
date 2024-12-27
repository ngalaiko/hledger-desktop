pub use hledger_parser::{ParseError, Posting, Transaction};

type TransactionFilter = Box<dyn for<'a> Fn(&'a Transaction) -> bool>;

type PostingFilter = Box<dyn for<'a> Fn(&'a Posting) -> bool>;

pub struct Query {
    transaction_filter: TransactionFilter,
    posting_filter: PostingFilter,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            transaction_filter: Box::new(|_| true),
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
        let transaction_filters = terms
            .iter()
            .map(to_transaction_filter)
            .collect::<Result<Vec<_>, _>>()?;
        let posting_filters = terms
            .iter()
            .map(to_posting_filter)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            transaction_filter: Box::new(move |tx| transaction_filters.iter().all(|f| f(tx))),
            posting_filter: Box::new(move |posting| posting_filters.iter().all(|f| f(posting))),
        })
    }

    #[must_use]
    pub fn filter(&self, tx: &Transaction) -> Option<Transaction> {
        if !(self.transaction_filter)(tx) {
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

fn to_transaction_filter(term: &hledger_parser::Term) -> Result<TransactionFilter, Error> {
    let is_not = term.is_not;
    match &term.condition {
        hledger_parser::Condition::Description(r) => {
            let r = regex::RegexBuilder::new(r)
                .case_insensitive(true)
                .build()
                .map_err(Error::Regex)?;
            Ok(Box::new(move |tx| {
                let is_match = if let Some(note) = &tx.note {
                    r.is_match(&format!("{} | {note}", tx.payee))
                } else {
                    r.is_match(&tx.payee)
                };
                (is_match && !is_not) || (!is_match && is_not)
            }))
        }
        _ => Ok(Box::new(|_| true)),
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
