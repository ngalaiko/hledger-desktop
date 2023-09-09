use crate::hledger::Commodity;

use super::accounts_tree;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct State {
    pub tree: accounts_tree::State,
    pub display_commodity: Option<Commodity>,
}
