use std::path::PathBuf;

use crate::hledger::include::parsers::parse_include_directive;

#[test]
fn test_parse_inlude_directive() {
    assert_eq!(
        parse_include_directive("include ./path/to/file.ext").unwrap(),
        ("", PathBuf::from("./path/to/file.ext"))
    );
}
