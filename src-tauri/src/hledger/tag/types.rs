use std::fmt::Display;

use serde::Serialize;


/// A tag and its possible value
/// 
/// # Example
/// 
/// ```
/// use hledger_parse::Tag;
/// 
/// let tag = Tag { name: "tag1".to_string(), value: None };
/// assert_eq!("tag1:", format!("{}", tag));
/// let tag = Tag { name: "tag1".to_string(), value: Some("some value".to_string()) };
/// assert_eq!("tag1:some value", format!("{}", tag));
/// ```
#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct Tag {
    pub name: String,
    pub value: Option<String>,
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value.as_ref() {
            Some(value) => write!(f, "{}:{}", self.name, value),
            None => write!(f, "{}:", self.name),
        }
    }
}
