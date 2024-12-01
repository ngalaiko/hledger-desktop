# hledger-desktop

attempt on implementing desktop app for viewing and maintaining hledger journals

## crates:

- [hledger-parser][] parser of individual hledger journal files
- [hledger-journal][] full hledger journal parser
- [hledger-desktop][] GUI implementation powered by [iced][]
- [hledger-desktop-ui][] GUI implementation powered by [egui][]
- [iced-virtual-list][] virtual list implementation for [iced][] copied from one of their wip branches

## goals

yet to be determined. first, i'd like to explore my journals visually.

probably want a crud to manually update them.

[hledger-desktop]: ./crates/hledger-desktop/
[hledger-desktop-ui]: ./crates/hledger-desktop-ui/
[hledger-journal]: ./crates/hledger-hournal/
[hledger-parser]: ./crates/hledger-parser/
[iced-virtual-list]: ./crates/iced-virtual-list/
[iced]: https://iced.rs
[egui]: https://egui.rs
