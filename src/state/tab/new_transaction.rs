use std::{collections::HashSet, path};

use chrono::{Local, NaiveDate};
use poll_promise::Promise;
use tauri::{AppHandle, Manager};

use crate::hledger;

pub struct State {
    creating: Option<Promise<Result<(), hledger::Error>>>,

    date: NaiveDate,
    description: String,
    postings: Vec<PostingState>,
    parsed_postings: Result<Vec<hledger::Posting>, Error>,
    destination: path::PathBuf,

    suggestions: Suggestions,
}

impl From<&Vec<hledger::Transaction>> for State {
    fn from(value: &Vec<hledger::Transaction>) -> Self {
        let suggestions = Suggestions::from(value);
        Self {
            creating: None,

            date: value
                .last()
                .map(|t| t.date)
                .unwrap_or_else(|| Local::now().date_naive()),
            description: String::new(),
            postings: vec![PostingState::default()],
            parsed_postings: Err(Error::InvalidPostings),
            destination: value
                .last()
                .map(|t| t.source_position.0.file_name.clone())
                .unwrap_or_else(|| {
                    suggestions
                        .destinations
                        .first()
                        .expect("at least one destination is always present")
                        .clone()
                }),
            suggestions,
        }
    }
}

impl State {
    pub fn is_loading(&self) -> bool {
        self.creating
            .as_ref()
            .map_or(false, |p| p.ready().is_none())
    }

    pub fn result(&self) -> Option<&Result<(), hledger::Error>> {
        self.creating.as_ref().and_then(|p| p.ready())
    }

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn suggestions(&self) -> &Suggestions {
        &self.suggestions
    }

    pub fn postings(&self) -> &[PostingState] {
        &self.postings
    }

    pub fn parsed_postings(&self) -> &Result<Vec<hledger::Posting>, Error> {
        &self.parsed_postings
    }

    pub fn destination(&self) -> &path::Path {
        &self.destination
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid postings")]
    InvalidPostings,
}

pub struct PostingState {
    account: String,
    parsed_account: Result<hledger::AccountName, hledger::ParseAccountNameError>,
    amount: String,
    parsed_amount: Result<hledger::Amount, hledger::ParseAmountError>,
}

impl Default for PostingState {
    fn default() -> Self {
        Self {
            account: String::new(),
            parsed_account: Err(hledger::ParseAccountNameError::Empty),
            amount: String::new(),
            parsed_amount: Ok(hledger::Amount::default()),
        }
    }
}

impl PostingState {
    pub fn account(&self) -> &str {
        &self.account
    }

    pub fn amount(&self) -> &str {
        &self.amount
    }

    pub fn parsed_amount(&self) -> &Result<hledger::Amount, hledger::ParseAmountError> {
        &self.parsed_amount
    }

    fn is_empty(&self) -> bool {
        self.account.is_empty() && self.amount.is_empty()
    }
}

pub enum Update {
    Ephemeral(Box<dyn Fn(&AppHandle, &mut State)>),
}

impl Update {
    pub fn set_date(date: &NaiveDate) -> Self {
        let date = *date;
        Self::Ephemeral(Box::new(move |_, state| {
            state.date = date;
        }))
    }

    pub fn set_description(description: &str) -> Self {
        let description = description.to_string();
        Self::Ephemeral(Box::new(move |_, state| {
            state.description = description.to_string();
        }))
    }

    pub fn set_posting_account(index: usize, account: &str) -> Self {
        let account = account.to_string();
        Self::Ephemeral(Box::new(move |_, state| {
            if let Some(posting) = state.postings.get_mut(index) {
                posting.account = account.to_string();
                posting.parsed_account = account.parse();
            }
        }))
        .and_then(Self::convert_postings())
        .and_then(Self::insert_new_postings())
    }

    pub fn set_posting_amount(index: usize, amount: &str) -> Self {
        let amount = amount.to_string();
        Self::Ephemeral(Box::new(move |_, state| {
            if let Some(posting) = state.postings.get_mut(index) {
                posting.amount = amount.to_string();
                posting.parsed_amount = amount.parse();
            }
        }))
        .and_then(Self::convert_postings())
        .and_then(Self::insert_new_postings())
    }

    pub fn set_destination(destination: &path::Path) -> Self {
        let destination = destination.to_path_buf();
        Self::Ephemeral(Box::new(move |_, state| {
            state.destination = destination.clone();
        }))
    }

    pub fn create_transaction(file_path: &path::Path, transaction: &hledger::Transaction) -> Self {
        let file_path = file_path.to_path_buf();
        let transaction = transaction.clone();
        Self::Ephemeral(Box::new(move |handle, state| {
            state.creating = Some(Promise::spawn_async({
                let manager = handle.state::<hledger::Manager>().inner().clone();
                let transaction = transaction.clone();
                let file_path = file_path.clone();
                async move {
                    let client = manager.client(file_path).await?;
                    client.add(&transaction).await
                }
            }))
        }))
    }

    fn convert_postings() -> Self {
        Self::Ephemeral(Box::new(|_, state| {
            state.parsed_postings = state
                .postings
                .iter()
                .filter(|posting| !posting.account.is_empty() || !posting.amount.is_empty())
                .map(|posting| {
                    match (
                        posting.parsed_account.as_ref(),
                        posting.parsed_amount.as_ref(),
                    ) {
                        (Ok(account), Ok(amount)) => Ok(hledger::Posting {
                            account: account.clone(),
                            amount: vec![amount.clone()],
                            ..Default::default()
                        }),
                        _ => Err(Error::InvalidPostings),
                    }
                })
                .collect::<Result<Vec<_>, _>>();
        }))
    }

    fn insert_new_postings() -> Self {
        Self::Ephemeral(Box::new(|_, state| {
            let empty_input_postings = state.postings.iter().filter(|p| p.is_empty()).count();

            if empty_input_postings == 0 {
                state.postings.push(PostingState::default())
            }
        }))
    }
}

impl Update {
    fn and_then(self, other: Self) -> Self {
        match (self, other) {
            (Self::Ephemeral(f), Self::Ephemeral(g)) => {
                Self::Ephemeral(Box::new(move |handle, s| {
                    f(handle, s);
                    g(handle, s);
                }))
            }
        }
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
            destinations,
            descriptions,
            account_names,
        }
    }
}
