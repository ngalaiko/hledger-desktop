use std::{fmt::Display, path::PathBuf};

use serde::Serialize;

use crate::hledger::{
    account::types::Account, commodity::types::Commodity, include::types::Include,
    price::types::Price, transaction::types::Transaction, HLParserError,
};

use super::parsers::parse_journal;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Value {
    Ignore,
    Include(Include),
    Transaction(Transaction),
    Included(Vec<Value>),
    Price(Price),
    Account(Account),
    Commodity(Commodity),
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct Journal {
    transactions: Vec<Transaction>,
    accounts: Vec<Account>,
    prices: Vec<Price>,
    commodities: Vec<Commodity>,
    includes: Vec<Include>,
}

impl TryFrom<PathBuf> for Journal {
    type Error = HLParserError;

    fn try_from(journal_path: PathBuf) -> Result<Self, HLParserError> {
        let base_path = journal_path.parent().map(|v| v.to_owned());
        let journal_contents = std::fs::read_to_string(journal_path)?;
        let journal = parse_journal(&journal_contents, base_path).map_err(|e| match e {
            HLParserError::Parse(i, ek) => HLParserError::Parse(i, ek),
            HLParserError::Validation(desc) => HLParserError::Validation(desc),
            HLParserError::IO(e) => HLParserError::IO(e),
            HLParserError::IncludePath(path) => HLParserError::IncludePath(path),
            HLParserError::Extract(v) => HLParserError::Extract(v),
        })?;

        Ok(journal)
    }
}

impl Display for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in &self.includes {
            writeln!(f, "include {}", i)?;
        }
        for a in &self.accounts {
            writeln!(f, "account {}", a)?;
        }
        for c in &self.commodities {
            writeln!(f, "{}", c)?;
        }
        for p in &self.prices {
            writeln!(f, "{}", p)?;
        }
        for t in &self.transactions {
            writeln!(f, "{}", t)?;
        }

        Ok(())
    }
}

impl Journal {
    pub fn new(
        transactions: Vec<Transaction>,
        accounts: Vec<Account>,
        prices: Vec<Price>,
        commodities: Vec<Commodity>,
        includes: Vec<Include>,
    ) -> Journal {
        Journal {
            includes,
            transactions,
            accounts,
            prices,
            commodities,
        }
    }

    pub fn includes(&self) -> Vec<Include> {
        self.includes.clone()
    }

    pub fn transactions(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }

    pub fn accounts(&self) -> Vec<Account> {
        self.accounts.clone()
    }

    pub fn prices(&self) -> Vec<Price> {
        self.prices.clone()
    }

    pub fn commodities(&self) -> Vec<Commodity> {
        self.commodities.clone()
    }

    pub fn payees(&self) -> Vec<String> {
        let mut tx_payees: Vec<String> = self
            .transactions
            .iter()
            .filter_map(|t| t.description.payee.clone())
            .collect();
        tx_payees.sort();
        let mut unique_payees: Vec<String> = vec![];

        tx_payees.iter().for_each(|p| {
            if !unique_payees.contains(p) {
                unique_payees.push(p.clone());
            }
        });

        unique_payees
    }

    pub fn validate_accounts(&self) -> Result<(), HLParserError> {
        let undefined_accounts: Vec<Account> = self
            .transactions
            .iter()
            .flat_map(|t| t.postings.clone())
            .map(|p| p.account)
            .filter(|a| !self.accounts.contains(a))
            .collect();
        if !undefined_accounts.is_empty() {
            return Err(HLParserError::Validation(format!(
                "The following accounts are not defined:\n{}",
                undefined_accounts
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            )));
        }
        Ok(())
    }
}
