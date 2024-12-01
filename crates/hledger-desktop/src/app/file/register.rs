use iced::{
    widget::{scrollable, text, Row},
    Element, Length, Task,
};
use iced_virtual_list::{Content, List};

#[derive(Debug)]
pub struct Register {
    content: Content<TransactionView>,
}

#[derive(Debug, Clone, Copy)]
pub struct Message;

impl Register {
    pub fn new() -> Self {
        Self {
            content: Content::new(),
        }
    }

    pub fn from_journal(journal: &hledger_journal::Journal) -> Self {
        let items = journal
            .transactions()
            .map(TransactionView::from_transaction)
            .collect();
        let content = Content::with_items(items);
        Self { content }
    }

    #[allow(clippy::unused_self)]
    pub fn update(&self, _message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        scrollable(List::new(&self.content, |_, tx| {
            let date = text!("{}", tx.date.format("%Y-%m-%d"));
            let payee = text!("{}", tx.payee.clone());
            let description = text!(
                "{}",
                tx.description
                    .clone()
                    .map(|d| format!("| {d}"))
                    .unwrap_or_default()
            );
            Row::with_children([date.into(), payee.into(), description.into()])
                .spacing(10)
                .width(Length::Fill)
                .into()
        }))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug)]
struct TransactionView {
    pub date: chrono::NaiveDate,
    pub payee: String,
    pub description: Option<String>,
}

impl TransactionView {
    pub fn from_transaction(tx: &hledger_journal::Transaction) -> Self {
        Self {
            date: tx.date,
            payee: tx.payee.clone(),
            description: tx.description.clone(),
        }
    }
}
