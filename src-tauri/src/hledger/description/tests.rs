use super::{parsers::parse_description, types::Description};

#[test]
fn test_parse_description() {
    assert_eq!(
        parse_description("some description").unwrap(),
        (
            "",
            Description {
                payee: None,
                note: Some("some description".into()),
            }
        )
    )
}

#[test]
fn test_parse_payee_and_note() {
    assert_eq!(
        parse_description("Acme | some description").unwrap(),
        (
            "",
            Description {
                payee: Some("Acme".into()),
                note: Some("some description".into()),
            }
        )
    )
}

#[test]
fn test_parse_empty_description() {
    assert_eq!(
        parse_description(" ").unwrap(),
        (
            "",
            Description {
                payee: None,
                note: None
            }
        )
    );
    assert_eq!(
        parse_description("").unwrap(),
        (
            "",
            Description {
                payee: None,
                note: None
            }
        )
    );
}

#[test]
fn test_parse_payee_and_note_different_spacing() {
    assert_eq!(
        parse_description("Acme| some description").unwrap(),
        (
            "",
            Description {
                payee: Some("Acme".into()),
                note: Some("some description".into()),
            }
        )
    );
    assert_eq!(
        parse_description("Acme |some description").unwrap(),
        (
            "",
            Description {
                payee: Some("Acme".into()),
                note: Some("some description".into()),
            }
        )
    );
    assert_eq!(
        parse_description("Acme|some description").unwrap(),
        (
            "",
            Description {
                payee: Some("Acme".into()),
                note: Some("some description".into()),
            }
        )
    );
}
