# hledger-parser

[![crates.io](https://img.shields.io/crates/v/hledger-parser.svg)](https://crates.io/crates/hledger-parser)
[![docs.rs](https://docs.rs/hledger-parser/badge.svg)](https://docs.rs/hledger-parser)
[![License](https://img.shields.io/crates/l/hledger-parser.svg)](https://raw.githubusercontent.com/ngalaiko/hledger-parser/refs/heads/master/LICENSE)

parser for hledger journals powered by [chumsky][]

## goals

* parse plaintext .journals into structured data to build tools on top

## non goals

* re-build hledger in rust

## current state

public beta

it's able to parse [cheatsheet][] and my personal ledger (which is quite extensive)

i don't like the api so far, so it will probably change

things i don't like:

* year directive - it is only used for parsing, probably no reason to export it?
* decimal mark directive - same thing
* error messages
* period::interval type definitions
* exporting chrono and rust_decimal types. maybe it's better to define own types?

## binary

a small binary comes with this lib that i found helpful during development and testing

it takes path to a .journal file, and outputs parse result or parsing error

```sh
> cargo run --features cli -- --help
Usage: hledger-parser --ledger-file <LEDGER_FILE>

Options:
      --ledger-file <LEDGER_FILE>  [env: LEDGER_FILE=/path/to/ledger.journal]
  -h, --help                       Print help
```

[chumsky]: https://github.com/zesterer/chumsky
[cheatsheet]: ./examples/fixture/cheatsheet.journal
