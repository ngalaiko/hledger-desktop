# hledger-desktop

attempt on implementing desktop app for viewing and maintaining hledger journals

## crates:

- [hledger-desktop][] GUI implementation powered by [egui][]
- [hledger-journal][] full hledger journal parser
- [hledger-parser][] parser of individual hledger journal files
- [hledger-query][] hledger query evaluator

## goals

yet to be determined. first, i'd like to explore my journals visually.

probably want a crud to manually update them.

[hledger-desktop]: ./crates/hledger-desktop/
[hledger-journal]: ./crates/hledger-journal/
[hledger-parser]: ./crates/hledger-parser/
[hledger-query]: ./crates/hledger-query/
[egui]: https://egui.rs
