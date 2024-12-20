mod account;
mod auto_postings;
mod commodity;
mod decimal_mark;
mod include;
mod payee;
mod price;
mod tag;
mod transaction;
mod year;

use chumsky::prelude::*;

use crate::component::comment::{block, inline, line};
use crate::component::whitespace::whitespace;
use crate::directive::account::account;
use crate::directive::auto_postings::auto_postings;
use crate::directive::commodity::commodity;
use crate::directive::decimal_mark::decimal_mark;
use crate::directive::include::include;
use crate::directive::payee::payee;
use crate::directive::price::price;
use crate::directive::tag::tag;
use crate::directive::year::year;
use crate::state::State;

pub use crate::directive::account::Account;
pub use crate::directive::auto_postings::{AutoPosting, AutosPostingRule};
pub use crate::directive::commodity::Commodity;
pub use crate::directive::decimal_mark::DecimalMark;
pub use crate::directive::include::{Format, Include};
pub use crate::directive::payee::Payee;
pub use crate::directive::price::Price;
pub use crate::directive::tag::Tag;
pub use crate::directive::transaction::{
    Assertion, Periodic as PeriodicTransaction, Posting, Simple as Transaction,
};
pub use crate::directive::year::Year;

#[derive(Clone, Debug)]
pub enum Directive {
    Account(Account),
    AutoPostings(AutosPostingRule),
    Commodity(Commodity),
    DecimalMark(DecimalMark),
    Include(Include),
    Payee(Payee),
    Price(Price),
    Tag(Tag),
    Transaction(Transaction),
    PeriodicTransaction(PeriodicTransaction),
    Year(Year),
}

pub fn directive<'a>() -> impl Parser<'a, &'a str, Directive, extra::Full<Rich<'a, char>, State, ()>>
{
    // .boxed() at the end of every choice option is important to not blow up compilation
    // complexity.
    //
    // see https://github.com/zesterer/chumsky/issues/13
    choice((
        account().map(Directive::Account).boxed(),
        auto_postings().map(Directive::AutoPostings).boxed(),
        commodity().map(Directive::Commodity).boxed(),
        decimal_mark().map(Directive::DecimalMark).boxed(),
        include().map(Directive::Include).boxed(),
        payee().map(Directive::Payee).boxed(),
        price().map(Directive::Price).boxed(),
        tag().map(Directive::Tag).boxed(),
        transaction::simple().map(Directive::Transaction).boxed(),
        transaction::periodic()
            .map(Directive::PeriodicTransaction)
            .boxed(),
        year().map(Directive::Year).boxed(),
    ))
}

pub fn directives<'a>(
) -> impl Parser<'a, &'a str, Vec<Directive>, extra::Full<Rich<'a, char>, State, ()>> {
    choice((
        directive().map(Some),
        inline().map(|_| None),
        line().map(|_| None),
        block().map(|_| None),
        whitespace().repeated().map(|()| None),
    ))
    .separated_by(text::newline())
    .collect::<Vec<_>>()
    .map(|directives| directives.into_iter().flatten().collect())
}
