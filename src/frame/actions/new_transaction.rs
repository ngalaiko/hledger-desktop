use std::{collections::HashSet, path};

use chrono::NaiveDate;
use poll_promise::Promise;
use tauri::Manager;

use crate::{
    frame::state::new_transaction::{Error, PostingState, State},
    hledger::{self, MixedAmount},
};

use super::action::StateAction;

pub type Update = StateAction<State>;

impl Update {
    pub fn set_date(date: NaiveDate) -> Self {
        Self::Ephemeral(Box::new(move |_, state| {
            state.date = date;
        }))
    }

    pub fn set_description(description: &str) -> Self {
        let description = description.to_string();
        Self::Ephemeral(Box::new(move |_, state| {
            state.description = description.clone();
        }))
    }

    pub fn set_posting_account(index: usize, account: &str) -> Self {
        let account = account.to_string();
        Self::Ephemeral(Box::new(move |_, state| {
            if let Some(posting) = state.postings.get_mut(index) {
                posting.account = account.clone();
                posting.parsed_account = account.parse();
            }
        }))
        .and_then(Self::insert_new_postings())
        .and_then(Self::convert_postings())
    }

    pub fn set_posting_amount(index: usize, amount: &str) -> Self {
        let amount = amount.to_string();
        Self::Ephemeral(Box::new(move |_, state| {
            if let Some(posting) = state.postings.get_mut(index) {
                posting.amount = amount.clone();
                posting.parsed_amount = amount.parse();
            }
        }))
        .and_then(Self::insert_new_postings())
        .and_then(Self::convert_postings())
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
            }));
        }))
    }

    fn convert_postings() -> Self {
        Self::Ephemeral(Box::new(|_, state| {
            let non_empty_postings = state.postings.iter().filter(|posting| {
                !matches!(
                    (
                        posting.parsed_account.as_ref(),
                        posting.parsed_amount.as_ref(),
                    ),
                    (
                        Err(hledger::ParseAccountNameError::Empty),
                        Err(hledger::ParseAmountError::Empty)
                    ),
                )
            });

            let empty_amounts = non_empty_postings
                .clone()
                .filter(|posting| {
                    matches!(
                        posting.parsed_amount.as_ref(),
                        Err(hledger::ParseAmountError::Empty),
                    )
                })
                .count();
            if empty_amounts > 1 {
                state.parsed_postings = Err(Error::TooManyEmptyAmounts);
                return;
            }

            let mut saldo = Some(
                non_empty_postings
                    .clone()
                    .filter_map(|posting| posting.parsed_amount.as_ref().ok())
                    .map(|amount| vec![amount.clone()].into())
                    .sum::<MixedAmount>()
                    .negate(),
            );

            let postings = non_empty_postings
                .map(|posting| {
                    match (
                        posting.parsed_account.as_ref(),
                        posting.parsed_amount.as_ref(),
                        saldo.as_ref(),
                    ) {
                        (Ok(account), Ok(amount), _) => Ok(hledger::Posting {
                            account: account.clone(),
                            amount: amount.into(),
                            ..Default::default()
                        }),
                        (Ok(account), Err(hledger::ParseAmountError::Empty), Some(s)) => {
                            let posting = hledger::Posting {
                                account: account.clone(),
                                amount: s.clone(),
                                ..Default::default()
                            };
                            saldo = None;
                            Ok(posting)
                        }
                        _ => Err(Error::InvalidPostings),
                    }
                })
                .collect::<Result<Vec<_>, _>>();

            if let Some(saldo) = saldo {
                if postings.is_ok() && !saldo.is_zero() {
                    state.parsed_postings = Err(Error::Unbalanced(saldo));
                    return;
                }
            }
            state.parsed_postings = postings;
        }))
    }

    fn insert_new_postings() -> Self {
        Self::Ephemeral(Box::new(|_, state| {
            let empty_input_postings = state.postings.iter().filter(|p| p.is_empty()).count();

            if empty_input_postings == 0 {
                state.postings.push(PostingState::default());
            }
        }))
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
