use std::fmt::Display;

use serde::Serialize;

/// Transaction description
/// 
/// This can hold a note about the transaction and the payee
/// 
/// # Example
/// 
/// ```
/// use hledger_parse::Description;
/// 
/// let description = Description { payee: None, note: Some("drinks".to_string()) };
/// assert_eq!("drinks", format!("{}", description));
/// let description = Description { payee: Some("Cheers bar".to_string()), note: Some("drinks".to_string()) };
/// assert_eq!("Cheers bar | drinks", format!("{}", description));
/// let description = Description { payee: Some("Cheers bar".to_string()), note: None };
/// assert_eq!("Cheers bar |", format!("{}", description));
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct Description {
    pub payee: Option<String>,
    pub note: Option<String>,
}

impl Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Description {
                payee: Some(p),
                note: Some(n),
            } => write!(f, "{} | {}", p, n),
            Description {
                payee: None,
                note: Some(n),
            } => write!(f, "{}", n),
            Description {
                payee: Some(p),
                note: None,
            } => write!(f, "{} |", p),
            Description {
                payee: None,
                note: None,
            } => write!(f, ""),
        }
    }
}

impl Description {
    pub fn is_missing(&self) -> bool {
        self.payee.is_none() && self.note.is_none()
    }
}
