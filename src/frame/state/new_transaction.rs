use std::{collections::HashSet, path};

use chrono::{Local, NaiveDate};

use crate::hledger;
use crate::promise::Promise;

pub struct State {
    pub creating: Promise<Result<(), hledger::Error>>,

    pub date: NaiveDate,
    pub description: String,
    pub postings: Vec<PostingState>,
    pub parsed_postings: Result<Vec<hledger::Posting>, Error>,
    pub destination: path::PathBuf,

    pub suggestions: Suggestions,
}

impl State {
    pub fn is_loading(&self) -> bool {
        self.creating.is_some() && self.creating.ready().is_none()
    }
}

impl From<&Vec<hledger::Transaction>> for State {
    fn from(value: &Vec<hledger::Transaction>) -> Self {
        let suggestions = Suggestions::from(value);
        Self {
            creating: Promise::default(),

            date: value
                .last()
                .map_or_else(|| Local::now().date_naive(), |t| t.date),
            description: String::new(),
            postings: vec![PostingState::default()],
            parsed_postings: Err(Error::InvalidPostings),
            destination: value.last().map_or_else(
                || {
                    suggestions
                        .destinations
                        .first()
                        .expect("at least one destination is always present")
                        .clone()
                },
                |t| t.source_position.0.file_name.clone(),
            ),
            suggestions,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid postings")]
    InvalidPostings,
    #[error("only one empty amount is allowed")]
    TooManyEmptyAmounts,
    #[error("unbalanced posting")]
    Unbalanced(hledger::MixedAmount),
}

#[derive(Debug)]
pub struct PostingState {
    pub account: String,
    pub parsed_account: Result<hledger::AccountName, hledger::ParseAccountNameError>,
    pub amount: String,
    pub parsed_amount: Result<hledger::Amount, hledger::ParseAmountError>,
}

impl Default for PostingState {
    fn default() -> Self {
        Self {
            account: String::new(),
            parsed_account: Err(hledger::ParseAccountNameError::Empty),
            amount: String::new(),
            parsed_amount: Err(hledger::ParseAmountError::Empty),
        }
    }
}

impl PostingState {
    pub fn is_empty(&self) -> bool {
        self.account.is_empty() && self.amount.is_empty()
    }
}

pub struct Suggestions {
    pub descriptions: Vec<String>,
    pub account_names: Vec<String>,
    pub destinations: Vec<path::PathBuf>,
}

impl From<&Vec<hledger::Transaction>> for Suggestions {
    fn from(transactions: &Vec<hledger::Transaction>) -> Self {
        let mut destinations = transactions
            .iter()
            .map(|a| a.source_position.0.file_name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        destinations.sort();
        let mut descriptions = transactions
            .iter()
            .map(|a| a.description.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        descriptions.sort();
        let mut account_names = transactions
            .iter()
            .flat_map(|a| a.postings.iter().map(|p| p.account.to_string().clone()))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        account_names.sort();
        Suggestions {
            descriptions,
            account_names,
            destinations,
        }
    }
}
