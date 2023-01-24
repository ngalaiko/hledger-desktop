use crate::hledger::status::{parsers::parse_status, types::Status};

#[test]
fn test_status_cleared() {
    assert_eq!(parse_status("* ").unwrap(), ("", Status::Cleared));
}

#[test]
fn test_status_pending() {
    assert_eq!(parse_status("! ").unwrap(), ("", Status::Pending));
}

#[test]
fn test_status_unmarked() {
    assert_eq!(parse_status("").unwrap(), ("", Status::Unmarked));
    assert_eq!(parse_status(" ").unwrap(), (" ", Status::Unmarked));
}
