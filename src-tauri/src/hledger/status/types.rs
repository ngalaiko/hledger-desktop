use std::fmt::Display;

use serde::Serialize;

/// Clearing status of a posting or a transaction
/// 
/// # Example
/// 
/// ```
/// use hledger_parse::Status;
/// 
/// assert_eq!("", format!("{}", Status::Unmarked));
/// assert_eq!("!", format!("{}", Status::Pending));
/// assert_eq!("*", format!("{}", Status::Cleared));
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub enum Status {
    Unmarked,
    Pending,
    Cleared,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Status::Unmarked => write!(f, ""),
            Status::Pending => write!(f, "!"),
            Status::Cleared => write!(f, "*"),
        }
    }
}
