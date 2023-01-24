use std::fmt::Display;

use serde::Serialize;

use crate::hledger::{journal::types::Value, HLParserError};

/// Declared commodity
/// 
/// # Example
/// 
/// ```
/// use hledger_parse::Commodity;
/// 
/// let commodity = Commodity { name: "INR".to_string(), format: None };
/// assert_eq!("commodity INR", format!("{}", commodity));
/// let commodity = Commodity { name: "INR".to_string(), format: Some("INR 1,00,00,000.00".to_string()) };
/// assert_eq!("commodity INR\n  format INR 1,00,00,000.00", format!("{}", commodity));
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct Commodity {
    pub name: String,
    pub format: Option<String>, // TODO: temp before I decide how to store the format properly
}

impl TryInto<Commodity> for Value {
    type Error = HLParserError;

    fn try_into(self) -> Result<Commodity, Self::Error> {
        if let Value::Commodity(c) = self {
            Ok(c)
        } else {
            Err(HLParserError::Extract(self))
        }
    }
}

impl Display for Commodity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.format.as_ref() {
            Some(format) => write!(f, "commodity {}\n  format {}", self.name, format),
            None => write!(f, "commodity {}", self.name),
        }
    }
}
