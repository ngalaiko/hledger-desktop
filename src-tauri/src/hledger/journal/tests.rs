use chrono::NaiveDate;
use rust_decimal_macros::dec;

use crate::hledger::{
    amount::types::Amount,
    description::types::Description,
    journal::{
        parsers::{parse_comment_value, parse_empty_line, parse_journal},
        types::Journal,
    },
    posting::types::Posting,
    status::types::Status,
    transaction::types::Transaction,
};

use super::{parsers::flatten_values, types::Value};

#[test]
fn test_parse_comment_value() {
    assert_eq!(
        parse_comment_value("; A sample journal file\n").unwrap(),
        ("", Value::Ignore)
    );
    assert_eq!(parse_comment_value(";\n").unwrap(), ("", Value::Ignore));
    assert_eq!(parse_comment_value(";").unwrap(), ("", Value::Ignore));
}

#[test]
fn test_parse_empty_line() {
    assert_eq!(parse_empty_line("\n").unwrap(), ("", Value::Ignore));
    assert_eq!(parse_empty_line("   \n").unwrap(), ("", Value::Ignore));
}

#[test]
fn test_flatten_values() {
    let values = vec![
        Value::Transaction(Transaction {
            primary_date: NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
            secondary_date: None,
            code: None,
            status: Status::Unmarked,
            description: Description {
                note: Some("income".into()),
                payee: None,
            },
            postings: vec![
                Posting {
                    account: "assets:bank:checking".into(),
                    amount: Some(Amount {
                        currency: "$".into(),
                        value: dec!(1),
                    }),
                    status: Status::Unmarked,
                    unit_price: None,
                    total_price: None,
                    balance_assertion: None,
                },
                Posting {
                    account: "income:salary".into(),
                    amount: None,
                    status: Status::Unmarked,
                    unit_price: None,
                    total_price: None,
                    balance_assertion: None,
                },
            ],
            tags: vec![],
        }),
        Value::Included(vec![
            Value::Transaction(Transaction {
                primary_date: NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
                secondary_date: None,
                code: None,
                status: Status::Unmarked,
                description: Description {
                    note: Some("income".into()),
                    payee: None,
                },
                postings: vec![
                    Posting {
                        account: "assets:bank:checking".into(),
                        amount: Some(Amount {
                            currency: "$".into(),
                            value: dec!(1),
                        }),
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                    Posting {
                        account: "income:salary".into(),
                        amount: None,
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                ],
                tags: vec![],
            }),
            Value::Included(vec![Value::Transaction(Transaction {
                primary_date: NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
                secondary_date: None,
                code: None,
                status: Status::Unmarked,
                description: Description {
                    note: Some("income".into()),
                    payee: None,
                },
                postings: vec![
                    Posting {
                        account: "assets:bank:checking".into(),
                        amount: Some(Amount {
                            currency: "$".into(),
                            value: dec!(1),
                        }),
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                    Posting {
                        account: "income:salary".into(),
                        amount: None,
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                ],
                tags: vec![],
            })]),
        ]),
    ];
    assert_eq!(
        flatten_values(values),
        [
            Value::Transaction(Transaction {
                primary_date: NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
                secondary_date: None,
                code: None,
                status: Status::Unmarked,
                description: Description {
                    note: Some("income".into()),
                    payee: None,
                },
                postings: vec![
                    Posting {
                        account: "assets:bank:checking".into(),
                        amount: Some(Amount {
                            currency: "$".into(),
                            value: dec!(1),
                        }),
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                    Posting {
                        account: "income:salary".into(),
                        amount: None,
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                ],
                tags: vec![],
            }),
            Value::Transaction(Transaction {
                primary_date: NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
                secondary_date: None,
                code: None,
                status: Status::Unmarked,
                description: Description {
                    note: Some("income".into()),
                    payee: None,
                },
                postings: vec![
                    Posting {
                        account: "assets:bank:checking".into(),
                        amount: Some(Amount {
                            currency: "$".into(),
                            value: dec!(1),
                        }),
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                    Posting {
                        account: "income:salary".into(),
                        amount: None,
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                ],
                tags: vec![],
            }),
            Value::Transaction(Transaction {
                primary_date: NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
                secondary_date: None,
                code: None,
                status: Status::Unmarked,
                description: Description {
                    note: Some("income".into()),
                    payee: None,
                },
                postings: vec![
                    Posting {
                        account: "assets:bank:checking".into(),
                        amount: Some(Amount {
                            currency: "$".into(),
                            value: dec!(1),
                        }),
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                    Posting {
                        account: "income:salary".into(),
                        amount: None,
                        status: Status::Unmarked,
                        unit_price: None,
                        total_price: None,
                        balance_assertion: None,
                    },
                ],
                tags: vec![],
            }),
        ]
    )
}

#[test]
fn test_parse_journal_simple() {
    // from https://github.com/simonmichael/hledger/blob/e9c19e12ef62d46f57d3cbbd6814dbcf04bbc508/examples/sample.journal
    let input = r#"; A sample journal file.
;
; Sets up this account tree:
; assets
;   bank
;     checking
;     saving
;   cash
; expenses
;   food
;   supplies
; income
;   gifts
;   salary
; liabilities
;   debts

; declare accounts:
; account assets:bank:checking
; account income:salary
; account income:gifts
; account assets:bank:saving
; account assets:cash
; account expenses:food
; account expenses:supplies
; account liabilities:debts

; declare commodities:
; commodity $

2008/01/01 income
    assets:bank:checking  $1
    income:salary

2008/06/01 gift
    assets:bank:checking  $1
    income:gifts

2008/06/02 save
    assets:bank:saving  $1
    assets:bank:checking

2008/06/03 * eat & shop
    expenses:food      $1
    expenses:supplies  $1
    assets:cash

2008/12/31 * pay off
    liabilities:debts  $1
    assets:bank:checking


;final comment
        "#;
    assert_eq!(
        parse_journal(input, None).unwrap(),
        Journal::new(
            vec![
                Transaction {
                    primary_date: NaiveDate::from_ymd_opt(2008, 1, 1).unwrap(),
                    secondary_date: None,
                    code: None,
                    status: Status::Unmarked,
                    description: Description {
                        note: Some("income".into()),
                        payee: None,
                    },
                    postings: vec![
                        Posting {
                            account: "assets:bank:checking".into(),
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                        Posting {
                            account: "income:salary".into(),
                            amount: None,
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                    ],
                    tags: vec![],
                },
                Transaction {
                    primary_date: NaiveDate::from_ymd_opt(2008, 6, 1).unwrap(),
                    secondary_date: None,
                    code: None,
                    status: Status::Unmarked,
                    description: Description {
                        note: Some("gift".into()),
                        payee: None,
                    },
                    postings: vec![
                        Posting {
                            account: "assets:bank:checking".into(),
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                        Posting {
                            account: "income:gifts".into(),
                            amount: None,
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                    ],
                    tags: vec![],
                },
                Transaction {
                    primary_date: NaiveDate::from_ymd_opt(2008, 6, 2).unwrap(),
                    secondary_date: None,
                    code: None,
                    status: Status::Unmarked,
                    description: Description {
                        note: Some("save".into()),
                        payee: None,
                    },
                    postings: vec![
                        Posting {
                            account: "assets:bank:saving".into(),
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                        Posting {
                            account: "assets:bank:checking".into(),
                            amount: None,
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                    ],
                    tags: vec![],
                },
                Transaction {
                    primary_date: NaiveDate::from_ymd_opt(2008, 6, 3).unwrap(),
                    secondary_date: None,
                    code: None,
                    status: Status::Cleared,
                    description: Description {
                        note: Some("eat & shop".into()),
                        payee: None,
                    },
                    postings: vec![
                        Posting {
                            account: "expenses:food".into(),
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                        Posting {
                            account: "expenses:supplies".into(),
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                        Posting {
                            account: "assets:cash".into(),
                            amount: None,
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                    ],
                    tags: vec![],
                },
                Transaction {
                    primary_date: NaiveDate::from_ymd_opt(2008, 12, 31).unwrap(),
                    secondary_date: None,
                    code: None,
                    status: Status::Cleared,
                    description: Description {
                        note: Some("pay off".into()),
                        payee: None,
                    },
                    postings: vec![
                        Posting {
                            account: "liabilities:debts".into(),
                            amount: Some(Amount {
                                currency: "$".into(),
                                value: dec!(1),
                            }),
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                        Posting {
                            account: "assets:bank:checking".into(),
                            amount: None,
                            status: Status::Unmarked,
                            unit_price: None,
                            total_price: None,
                            balance_assertion: None,
                        },
                    ],
                    tags: vec![],
                },
            ],
            vec![],
            vec![],
            vec![],
            vec![]
        )
    );
}
