use std::fmt::Display;

use serde::Serialize;

use crate::{hledger::journal::types::Value, HLParserError};

#[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize)]
pub struct Account(String);

impl Account {
    pub fn components(&self) -> Vec<String> {
        self.0.split(':').map(|v| v.to_string()).collect()
    }

    pub fn is_child_of(&self, account: &Account) -> bool {
        self.0.starts_with(&account.to_string())
    }
}

impl TryInto<Account> for Value {
    type Error = HLParserError;

    fn try_into(self) -> Result<Account, Self::Error> {
        if let Value::Account(t) = self {
            Ok(t)
        } else {
            Err(HLParserError::Extract(self))
        }
    }
}

impl From<String> for Account {
    fn from(value: String) -> Self {
        Account(value)
    }
}

impl From<&str> for Account {
    fn from(value: &str) -> Self {
        Account(value.into())
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
