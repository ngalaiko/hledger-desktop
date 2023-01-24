use std::fmt::Display;

use serde::Serialize;

use crate::hledger::{amount::types::Amount, status::types::Status, Account};

/// Posting of a transaction
/// 
/// # Example
/// 
/// ```
/// use rust_decimal_macros::dec;
/// use hledger_parse::{Posting, Status, Amount};
/// 
/// let posting = Posting {
///     status: Status::Pending,
///     account: "expenses:food".into(),
///     amount: Some(Amount {
///         currency: "EUR".to_string(),
///         value: dec!(100)
///     }),
///     unit_price: None,
///     total_price: None,
/// };
/// assert_eq!("  ! expenses:food  100 EUR", format!("{}", posting));
/// let posting = Posting {
///     status: Status::Pending,
///     account: "expenses:food".into(),
///     amount: Some(Amount {
///         currency: "EUR".to_string(),
///         value: dec!(100)
///     }),
///     unit_price: Some(Amount {
///         currency: "USD".to_string(),
///         value: dec!(1.05)
///     }),
///     total_price: None,
/// };
/// assert_eq!("  ! expenses:food  100 EUR @ 1.05 USD", format!("{}", posting));
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct Posting {
    pub status: Status,
    pub account: Account,
    pub amount: Option<Amount>,
    pub unit_price: Option<Amount>,
    pub total_price: Option<Amount>,
}

impl Display for Posting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.amount.as_ref(), self.unit_price.as_ref(), self.total_price.as_ref()) {
            (None, None, None) => write!(f, "  {} {}", self.status, self.account),
            (Some(amount), None, None) => write!(f, "  {} {}  {}", self.status, self.account, amount),
            (Some(amount), Some(unit_price), None) => write!(f, "  {} {}  {} @ {}", self.status, self.account, amount, unit_price),
            (Some(amount), None, Some(total_price)) => write!(f, "  {} {}  {} @@ {}", self.status, self.account, amount, total_price),
            _ => unreachable!()
        }
    }
}

#[derive(Clone)]
pub struct PostingComplexAmount {
    pub amount: Option<Amount>,
    pub unit_price: Option<Amount>,
    pub total_price: Option<Amount>,
}
