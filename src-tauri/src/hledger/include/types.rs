use std::{fmt::Display, path::PathBuf};

use serde::Serialize;

use crate::{hledger::journal::types::Value, HLParserError};

#[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize)]
pub struct Include(PathBuf);

impl TryInto<Include> for Value {
    type Error = HLParserError;

    fn try_into(self) -> Result<Include, Self::Error> {
        if let Value::Include(t) = self {
            Ok(t)
        } else {
            Err(HLParserError::Extract(self))
        }
    }
}

impl From<PathBuf> for Include {
    fn from(value: PathBuf) -> Self {
        Include(value)
    }
}

impl From<String> for Include {
    fn from(value: String) -> Self {
        Include(PathBuf::from(value))
    }
}

impl From<&str> for Include {
    fn from(value: &str) -> Self {
        Include(PathBuf::from(value))
    }
}

impl Display for Include {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
