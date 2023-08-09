use std::collections::HashSet;

use crate::hledger;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
    pub checked: HashSet<hledger::AccountName>,
    pub open: HashSet<hledger::AccountName>,
}
