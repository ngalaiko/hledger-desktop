#![allow(clippy::assigning_clones)]

use std::collections::HashSet;

use futures::FutureExt;
use tokio::join;

use crate::{
    converter::Converter,
    frame::state::{
        new_transaction as new_transaction_state,
        tab::{AccountTreeNode, State},
    },
    hledger,
    promise::Promise,
};

use super::{action::StateAction, new_transaction};

pub type Update = StateAction<State>;

impl From<new_transaction::Update> for Update {
    fn from(value: new_transaction::Update) -> Self {
        match value {
            new_transaction::Update::Ephemeral(update) => {
                Update::Ephemeral(Box::new(move |manager, tab_state| {
                    if let Some(ref mut new_transaction_modal_state) =
                        tab_state.new_transaction_modal
                    {
                        update(manager, new_transaction_modal_state);
                    }
                }))
            }
            new_transaction::Update::Persistent(update) => {
                Update::Persistent(Box::new(move |manager, tab_state| {
                    if let Some(ref mut new_transaction_modal_state) =
                        tab_state.new_transaction_modal
                    {
                        update(manager, new_transaction_modal_state);
                    }
                }))
            }
        }
    }
}

impl Update {
    pub fn open_new_transaction_modal() -> Self {
        Update::Ephemeral(Box::new(move |_, tab_state| {
            if let Some(transactions) = match tab_state.transactions.ready() {
                None | Some(Err(_)) => None,
                Some(Ok(transactions)) => Some(transactions),
            } {
                tab_state.new_transaction_modal =
                    Some(new_transaction_state::State::from(transactions));
            }
        }))
    }

    pub fn close_new_transaction_modal() -> Self {
        Update::Ephemeral(Box::new(move |_, tab_state| {
            tab_state.new_transaction_modal = None;
        }))
    }

    pub fn check_account(node: &AccountTreeNode) -> Self {
        let account_name = node.name().clone();
        let siblings = node.siblings.clone();
        Update::Persistent(Box::new(move |_, tab_state| {
            tab_state
                .unchecked_accounts
                .retain(|a| !account_name.is_parent_of(a));
            tab_state.unchecked_accounts.remove(&account_name);
            if let Some(parent) = account_name.parent() {
                if tab_state.unchecked_accounts.contains(&parent) {
                    tab_state.unchecked_accounts.remove(&parent);
                    for sibling in &siblings {
                        tab_state.unchecked_accounts.insert(sibling.clone());
                    }
                }
            }
        }))
        .and_then(Update::recalculate_account_trees())
        .and_then(Update::recalculate_display_transactions())
    }

    pub fn uncheck_account(node: &AccountTreeNode) -> Self {
        let account_name = node.name().clone();
        Update::Persistent(Box::new(move |_, tab_state| {
            tab_state
                .unchecked_accounts
                .retain(|a| !account_name.is_parent_of(a));
            tab_state.unchecked_accounts.insert(account_name.clone());
        }))
        .and_then(Update::recalculate_account_trees())
        .and_then(Update::recalculate_display_transactions())
    }

    pub fn expand_account(account_name: &hledger::AccountName) -> Self {
        let account_name = account_name.clone();
        Update::Persistent(Box::new(move |_, tab_state| {
            tab_state.expanded_accounts.insert(account_name.clone());
        }))
        .and_then(Update::recalculate_account_trees())
    }

    pub fn collapse_account(account_name: &hledger::AccountName) -> Self {
        let account_name = account_name.clone();
        Update::Persistent(Box::new(move |_, tab_state| {
            tab_state
                .expanded_accounts
                .retain(|a| !a.eq(&account_name) && !account_name.is_parent_of(a));
        }))
        .and_then(Update::recalculate_account_trees())
    }

    pub fn set_display_commodity(commodity: Option<&hledger::Commodity>) -> Self {
        let commodity = commodity.cloned();
        Update::Persistent(Box::new(move |_, tab_state| {
            tab_state.display_commodity = commodity.clone();
        }))
        .and_then(Update::recalculate_display_transactions())
    }

    pub fn load_commodities() -> Self {
        Update::Ephemeral(Box::new(move |manager, tab_state| {
            if tab_state.commodities.is_some() {
                return;
            }
            let file_path = tab_state.file_path.clone();
            let manager = manager.clone();
            tab_state.commodities = Promise::spawn_async({
                async move {
                    let client = manager.client(file_path).await?;
                    let commodities = client.commodities().await?;
                    let commodities = commodities
                        .into_iter()
                        .filter(|c| !c.eq("AUTO"))
                        .collect::<Vec<_>>();
                    Ok(commodities)
                }
            });
        }))
    }

    pub fn reload_transactions() -> Self {
        Update::Ephemeral(Box::new(move |_, tab_state| {
            tab_state.transactions = Promise::default();
            tab_state.display_transactions = Promise::default();
        }))
        .and_then(Update::load_transactions())
    }

    pub fn load_transactions() -> Self {
        Update::Ephemeral(Box::new(move |manager, tab_state| {
            if tab_state.transactions.is_some() {
                return;
            }

            let load_prices_future = {
                let file_path = tab_state.file_path.clone();
                let manager = manager.clone();
                async move {
                    let client = manager.client(file_path).await?;
                    let prices = client.prices().await?;
                    Ok(prices)
                }
            }
            .shared();
            tab_state.prices = Promise::spawn_async(load_prices_future.clone());

            let load_transactions_future = {
                let file_path = tab_state.file_path.clone();
                let manager = manager.clone();
                async move {
                    let client = manager.client(file_path).await?;
                    let transactions = client.transactions().await?;
                    Ok(transactions)
                }
            }
            .shared();
            tab_state.transactions = Promise::spawn_async(load_transactions_future.clone());

            let load_converter_future = {
                let load_transactions_future = load_transactions_future.clone();
                async move {
                    let (prices, transactions) =
                        join!(load_prices_future, load_transactions_future);
                    let (prices, transactions) = (prices?, transactions?);
                    let converter = Converter::new(&prices, &transactions);
                    Ok(converter)
                }
            }
            .shared();
            tab_state.converter = Promise::spawn_async(load_converter_future.clone());

            tab_state.display_transactions = Promise::spawn_async({
                let unchecked_accounts = tab_state.unchecked_accounts.clone();
                let display_commodity = tab_state.display_commodity.clone();
                async move {
                    let (converter, transactions) =
                        join!(load_converter_future, load_transactions_future);
                    let (converter, transactions) = (converter?, transactions?);
                    let transactions = transactions
                        .iter()
                        .filter_map(|transaction| {
                            to_display_transaction(
                                transaction,
                                &converter,
                                &unchecked_accounts,
                                display_commodity.as_ref(),
                            )
                        })
                        .collect::<Vec<_>>();
                    Ok(transactions)
                }
            });
        }))
    }

    pub fn load_account_trees() -> Self {
        Update::Ephemeral(Box::new(move |manager, tab_state| {
            if tab_state.accounts_tree.is_some() {
                return;
            }

            let load_accounts_future = {
                let file_path = tab_state.file_path.clone();
                let manager = manager.clone();
                async move {
                    let client = manager.client(file_path).await?;
                    let accounts = client.accounts().await?;
                    Ok(accounts)
                }
            }
            .shared();

            tab_state.accounts = Promise::spawn_async(load_accounts_future.clone());
            tab_state.accounts_tree = Promise::spawn_async({
                let expanded_accounts = tab_state.expanded_accounts.clone();
                let unchecked_accounts = tab_state.unchecked_accounts.clone();
                async move {
                    let accounts = load_accounts_future.await?;
                    let trees = AccountTreeNode::new(
                        None,
                        &accounts,
                        &expanded_accounts,
                        &unchecked_accounts,
                    );
                    Ok(trees)
                }
            });
        }))
    }

    fn recalculate_display_transactions() -> Self {
        Update::Ephemeral(Box::new(|_, tab_state| {
            let transactions = match tab_state.transactions.ready() {
                None | Some(Err(_)) => None,
                Some(Ok(transactions)) => Some(transactions.clone()),
            };

            let converter = match tab_state.converter.ready() {
                None | Some(Err(_)) => None,
                Some(Ok(converter)) => Some(converter.clone()),
            };

            if let (Some(trasnsactions), Some(converter)) = (transactions, converter) {
                let unchecked_accounts = tab_state.unchecked_accounts.clone();
                let display_commodity = tab_state.display_commodity.clone();
                tab_state.display_transactions = Promise::spawn_blocking(move || {
                    let display_transactions = trasnsactions
                        .iter()
                        .filter_map(|transaction| {
                            to_display_transaction(
                                transaction,
                                &converter,
                                &unchecked_accounts,
                                display_commodity.as_ref(),
                            )
                        })
                        .collect::<Vec<_>>();
                    Ok(display_transactions)
                });
            }
        }))
    }

    fn recalculate_account_trees() -> Self {
        Update::Ephemeral(Box::new(|_, tab_state| {
            if let Some(accounts) = match tab_state.accounts.ready() {
                None | Some(Err(_)) => None,
                Some(Ok(accounts)) => Some(accounts),
            } {
                let trees = AccountTreeNode::new(
                    None,
                    accounts,
                    &tab_state.expanded_accounts,
                    &tab_state.unchecked_accounts,
                );
                tab_state.accounts_tree = Promise::from_ready(Ok(trees));
            }
        }))
    }
}

fn to_display_transaction(
    transaction: &hledger::Transaction,
    converter: &Converter,
    unchecked_accounts: &HashSet<hledger::AccountName>,
    display_commotidy: Option<&hledger::Commodity>,
) -> Option<hledger::Transaction> {
    let postings = transaction
        .postings
        .iter()
        .filter(|posting| {
            !unchecked_accounts.contains(&posting.account)
                && !posting
                    .account
                    .parents()
                    .iter()
                    .any(|parent| unchecked_accounts.contains(parent))
        })
        .map(|posting| hledger::Posting {
            amount: posting
                .amount
                .iter()
                .map(|amount| {
                    display_commotidy.map_or_else(
                        || amount.clone(),
                        |display_commotidy| {
                            // TODO: try to use amount's price here
                            converter
                                .convert(amount, display_commotidy, transaction.date)
                                .unwrap_or_else(|_| amount.clone())
                        },
                    )
                })
                .collect::<Vec<_>>()
                .into(),
            ..posting.clone()
        })
        .collect::<Vec<_>>();

    if postings.is_empty() {
        None
    } else {
        Some(hledger::Transaction {
            postings,
            ..transaction.clone()
        })
    }
}
