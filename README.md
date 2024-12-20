# hledger-desktop

attempt on implementing desktop app for viewing and maintaining hledger journals

## crates:

- [hledger-parser][] parser of individual hledger journal files
- [hledger-journal][] full hledger journal parser
- [hledger-desktop-ui][] GUI implementation powered by [egui][]

## goals

yet to be determined. first, i'd like to explore my journals visually.

probably want a crud to manually update them.

[hledger-desktop-ui]: ./crates/hledger-desktop-ui/
[hledger-journal]: ./crates/hledger-journal/
[hledger-parser]: ./crates/hledger-parser/
[iced]: https://iced.rs
[egui]: https://egui.rs
