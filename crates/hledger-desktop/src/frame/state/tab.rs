use std::{collections::HashSet, path};

use serde::{Deserialize, Serialize};

use crate::{
    converter::Converter,
    hledger::{self, AccountName, Commodity, Transaction},
    promise::Promise,
    widgets::CheckboxState,
};

use super::new_transaction;

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub file_path: path::PathBuf,

    pub expanded_accounts: HashSet<hledger::AccountName>,
    pub unchecked_accounts: HashSet<hledger::AccountName>,

    pub display_commodity: Option<hledger::Commodity>,

    #[serde(skip)]
    pub accounts: Promise<Result<Vec<hledger::Account>, hledger::Error>>,
    #[serde(skip)]
    pub transactions: Promise<Result<Vec<Transaction>, hledger::Error>>,
    #[serde(skip)]
    pub commodities: Promise<Result<Vec<Commodity>, hledger::Error>>,
    #[serde(skip)]
    pub prices: Promise<Result<Vec<hledger::Price>, hledger::Error>>,

    #[serde(skip)]
    pub display_transactions: Promise<Result<Vec<Transaction>, hledger::Error>>,
    #[serde(skip)]
    pub accounts_tree: Promise<Result<AccountTreeNode, hledger::Error>>,
    #[serde(skip)]
    pub converter: Promise<Result<Converter, hledger::Error>>,
    #[serde(skip)]
    pub new_transaction_modal: Option<new_transaction::State>,
}

impl State {
    pub fn name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }
}

impl From<path::PathBuf> for State {
    fn from(value: path::PathBuf) -> Self {
        Self {
            file_path: value.clone(),
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct AccountTreeNode {
    account_name: hledger::AccountName,
    children: Vec<AccountTreeNode>,
    pub siblings: Vec<hledger::AccountName>,
    is_expanded: bool,
    checkbox_state: CheckboxState,
}

impl AccountTreeNode {
    pub fn new(
        root: Option<&hledger::Account>,
        all_accounts: &[hledger::Account],
        expanded_accounts: &HashSet<hledger::AccountName>,
        unchecked_accounts: &HashSet<hledger::AccountName>,
    ) -> Self {
        let root = root.unwrap_or_else(|| {
            all_accounts
                .iter()
                .find(|a| a.name.to_string() == "root")
                .expect("root account is always present or provided")
        });

        let children = all_accounts
            .iter()
            .filter(|a| a.parent.eq(&root.name))
            .map(|account| {
                Self::new(
                    Some(account),
                    all_accounts,
                    expanded_accounts,
                    unchecked_accounts,
                )
            })
            .collect::<Vec<AccountTreeNode>>();

        let siblings = all_accounts
            .iter()
            .filter(|a| a.parent.eq(&root.parent))
            .filter(|a| a.name != root.name)
            .map(|account| account.name.clone())
            .collect::<Vec<_>>();

        let is_expanded = expanded_accounts.contains(&root.name);
        let checkbox_state = if unchecked_accounts.contains(&root.name)
            || root
                .name
                .parents()
                .iter()
                .any(|parent| unchecked_accounts.contains(parent))
        {
            CheckboxState::Unchecked
        } else if children.is_empty() {
            CheckboxState::Checked
        } else {
            let children_states = children
                .iter()
                .map(|child| child.checkbox_state)
                .collect::<Vec<CheckboxState>>();

            if children_states
                .iter()
                .all(|state| state == &CheckboxState::Checked)
            {
                CheckboxState::Checked
            } else if children_states
                .iter()
                .all(|state| state == &CheckboxState::Unchecked)
            {
                CheckboxState::Unchecked
            } else {
                CheckboxState::Indeterminate
            }
        };

        Self {
            account_name: root.name.clone(),
            children,
            siblings,
            is_expanded,
            checkbox_state,
        }
    }

    pub fn is_expanded(&self) -> &bool {
        &self.is_expanded
    }

    pub fn checkbox_state(&self) -> &CheckboxState {
        &self.checkbox_state
    }

    pub fn name(&self) -> &AccountName {
        &self.account_name
    }

    pub fn children(&self) -> &[AccountTreeNode] {
        &self.children
    }
}
