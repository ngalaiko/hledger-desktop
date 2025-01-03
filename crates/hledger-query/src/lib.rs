pub use hledger_parser::{ParseError, Posting, Transaction};

#[derive(Default)]
pub struct Query {
    description_filters: Vec<Filter>,
    account_filters: Vec<Filter>,
    status_filters: Vec<Filter>,
    other_filters: Vec<Filter>,
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
        let description_filters = terms
            .iter()
            .filter(|term| matches!(term.condition, hledger_parser::Condition::Description(_)))
            .map(to_filter)
            .collect::<Result<Vec<_>, _>>()?;
        let account_filters = terms
            .iter()
            .filter(|term| matches!(term.condition, hledger_parser::Condition::Account(_)))
            .map(to_filter)
            .collect::<Result<Vec<_>, _>>()?;
        let status_filters = terms
            .iter()
            .filter(|term| matches!(term.condition, hledger_parser::Condition::Status(_)))
            .map(to_filter)
            .collect::<Result<Vec<_>, _>>()?;
        let other_filters = terms
            .iter()
            .filter(|term| {
                !matches!(
                    term.condition,
                    hledger_parser::Condition::Description(_)
                        | hledger_parser::Condition::Account(_)
                        | hledger_parser::Condition::Status(_)
                )
            })
            .map(to_filter)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            description_filters,
            account_filters,
            status_filters,
            other_filters,
        })
    }

    /// When given multiple query terms, this will match:
    ///   * any of the description terms AND
    ///   * any of the account terms AND
    ///   * any of the status terms AND
    ///   * all the other terms.
    #[must_use]
    pub fn filter(&self, tx: &Transaction) -> Option<Transaction> {
        if !self.filter_transaction(tx) {
            return None;
        }

        let postings = tx
            .postings
            .iter()
            .filter(|posting| self.filter_posting(posting))
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

    fn filter_posting(&self, posting: &Posting) -> bool {
        (self.account_filters.is_empty()
            || self
                .account_filters
                .iter()
                .any(|f| f.filter_posting(posting)))
            && (self.other_filters.is_empty()
                || self.other_filters.iter().all(|f| f.filter_posting(posting)))
    }

    fn filter_transaction(&self, tx: &Transaction) -> bool {
        (self.description_filters.is_empty()
            || self
                .description_filters
                .iter()
                .any(|f| f.filter_transaction(tx)))
            && (self.status_filters.is_empty()
                || self.status_filters.iter().any(|f| f.filter_transaction(tx)))
            && (self.other_filters.is_empty()
                || self.other_filters.iter().all(|f| f.filter_transaction(tx)))
    }
}

enum Filter {
    Transaction(transaction::Filter),
    Posting(posting::Filter),
}

impl Filter {
    pub fn filter_transaction(&self, tx: &Transaction) -> bool {
        match self {
            Self::Posting(_) => true,
            Self::Transaction(filter) => filter(tx),
        }
    }

    pub fn filter_posting(&self, posting: &Posting) -> bool {
        match self {
            Self::Posting(filter) => filter(posting),
            Self::Transaction(_) => true,
        }
    }
}

fn to_filter(term: &hledger_parser::Term) -> Result<Filter, Error> {
    let filter = match &term.condition {
        hledger_parser::Condition::Account(query) => {
            posting::account_filter(query).map(Filter::Posting)
        }
        hledger_parser::Condition::Currency(query) => posting::currency(query).map(Filter::Posting),
        hledger_parser::Condition::Amount(filter) => Ok(Filter::Posting(posting::amount(filter))),
        hledger_parser::Condition::Code(query) => {
            transaction::code_filter(query).map(Filter::Transaction)
        }
        hledger_parser::Condition::Description(query) => {
            transaction::description_filter(query).map(Filter::Transaction)
        }
        hledger_parser::Condition::Note(query) => {
            transaction::note_filter(query).map(Filter::Transaction)
        }
        hledger_parser::Condition::Payee(query) => {
            transaction::payee_filter(query).map(Filter::Transaction)
        }
        hledger_parser::Condition::Date(period) => {
            Ok(Filter::Transaction(transaction::date_filter(period)))
        }
        hledger_parser::Condition::Status(status) => Ok(Filter::Transaction(
            transaction::status_filter(status.as_ref()),
        )),
    }?;
    if term.is_not {
        match filter {
            Filter::Transaction(filter) => Ok(Filter::Transaction(transaction::not_filter(filter))),
            Filter::Posting(filter) => Ok(Filter::Posting(posting::not_filter(filter))),
        }
    } else {
        Ok(filter)
    }
}

mod posting {
    use super::{Error, Posting};

    pub type Filter = Box<dyn for<'a> Fn(&'a Posting) -> bool>;

    pub fn account_filter(query: &str) -> Result<Filter, Error> {
        let r = regex::RegexBuilder::new(query)
            .case_insensitive(true)
            .build()
            .map_err(Error::Regex)?;
        Ok(Box::new(move |posting| {
            r.is_match(&posting.account_name.to_string())
        }))
    }

    pub fn currency(query: &str) -> Result<Filter, Error> {
        let r = regex::RegexBuilder::new(query)
            .case_insensitive(true)
            .build()
            .map_err(Error::Regex)?;
        Ok(Box::new(move |posting| {
            if let Some(amount) = &posting.amount {
                r.is_match(&amount.commodity)
            } else {
                false
            }
        }))
    }

    pub fn amount(filter: &hledger_parser::AmountCondition) -> Filter {
        let filter = filter.clone();
        Box::new(move |posting| match &filter {
            hledger_parser::AmountCondition::Equal(sign, mut amount) => {
                let Some(posting_amount) = &posting.amount else {
                    return false;
                };
                match sign {
                    None => posting_amount.quantity.abs() == amount,
                    Some(hledger_parser::AmountSign::Minus) => {
                        amount.set_sign_negative(true);
                        posting_amount.quantity == amount
                    }
                    Some(hledger_parser::AmountSign::Plus) => posting_amount.quantity == amount,
                }
            }
            hledger_parser::AmountCondition::Less(sign, mut amount) => {
                let Some(posting_amount) = &posting.amount else {
                    return false;
                };
                match sign {
                    None => posting_amount.quantity.abs() < amount,
                    Some(hledger_parser::AmountSign::Minus) => {
                        amount.set_sign_negative(true);
                        posting_amount.quantity < amount
                    }
                    Some(hledger_parser::AmountSign::Plus) => posting_amount.quantity < amount,
                }
            }
            hledger_parser::AmountCondition::Greater(sign, mut amount) => {
                let Some(posting_amount) = &posting.amount else {
                    return false;
                };
                match sign {
                    None => posting_amount.quantity.abs() > amount,
                    Some(hledger_parser::AmountSign::Minus) => {
                        amount.set_sign_negative(true);
                        posting_amount.quantity > amount
                    }
                    Some(hledger_parser::AmountSign::Plus) => posting_amount.quantity > amount,
                }
            }
            hledger_parser::AmountCondition::GreaterOrEqual(sign, mut amount) => {
                let Some(posting_amount) = &posting.amount else {
                    return false;
                };
                match sign {
                    None => posting_amount.quantity.abs() >= amount,
                    Some(hledger_parser::AmountSign::Minus) => {
                        amount.set_sign_negative(true);
                        posting_amount.quantity >= amount
                    }
                    Some(hledger_parser::AmountSign::Plus) => posting_amount.quantity >= amount,
                }
            }
            hledger_parser::AmountCondition::LessOrEqual(sign, mut amount) => {
                let Some(posting_amount) = &posting.amount else {
                    return false;
                };
                match sign {
                    None => posting_amount.quantity.abs() <= amount,
                    Some(hledger_parser::AmountSign::Minus) => {
                        amount.set_sign_negative(true);
                        posting_amount.quantity <= amount
                    }
                    Some(hledger_parser::AmountSign::Plus) => posting_amount.quantity <= amount,
                }
            }
        })
    }

    pub fn not_filter(filter: Filter) -> Filter {
        Box::new(move |tx| !filter(tx))
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
