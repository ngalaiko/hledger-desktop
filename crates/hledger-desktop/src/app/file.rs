use std::collections::HashSet;

use hledger_journal::Journal;
use iced::futures::channel::mpsc;
use iced::futures::SinkExt;
use iced::widget::{scrollable, text, Row};
use iced::{Element, Length, Subscription, Task};

use crate::promise::Promise;
use crate::watcher;

use iced_virtual_list::{Content, List};

#[derive(Debug)]
pub struct File {
    pub path: std::path::PathBuf,
    watcher_input: Option<mpsc::Sender<watcher::Input>>,
    journal: Promise<Result<hledger_journal::Journal, hledger_journal::Error>>,
    content: Content<hledger_journal::Transaction>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Watcher(watcher::Message),
    Loaded(Result<hledger_journal::Journal, hledger_journal::Error>),
    Updated(Result<hledger_journal::Journal, hledger_journal::Error>),
}

impl File {
    pub fn new(path: std::path::PathBuf) -> (Self, Task<Message>) {
        (
            Self {
                path: path.clone(),
                watcher_input: None,
                journal: Promise::Loading,
                content: Content::new(),
            },
            Task::perform(hledger_journal::Journal::load(path), Message::Loaded),
        )
    }

    #[allow(clippy::unused_self)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Updated(result) => match (&mut self.journal, &result) {
                (Promise::Loaded(Ok(journal)), Ok(updated)) => {
                    journal.merge(updated);
                    Task::none()
                }
                (_, Err(error)) => {
                    tracing::error!(?error, "failed to update journal");
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Loaded(result) => {
                let task = match &result {
                    Ok(journal) => {
                        let mut watcher_input = self
                            .watcher_input
                            .clone()
                            .expect("watcher initialized before file is selected");
                        let journal_includes = journal.includes();
                        self.content =
                            Content::with_items(journal.transactions().cloned().collect());
                        Task::future(async move {
                            watcher_input
                                .send(watcher::Input::Watch(journal_includes))
                                .await
                        })
                        .then(|_| Task::none())
                    }
                    Err(error) => {
                        tracing::error!(?error, "failed to load journal");
                        Task::none()
                    }
                };
                self.journal = Promise::Loaded(result);
                task
            }
            Message::Watcher(watcher::Message::Started(watcher_input)) => {
                assert!(self.watcher_input.is_none(), "watcher started twice");
                self.watcher_input.replace(watcher_input);
                Task::none()
            }
            Message::Watcher(watcher::Message::FileChange(paths)) => match &self.journal {
                Promise::Loaded(Ok(journal)) => {
                    let journal_includes = journal.includes().into_iter().collect::<HashSet<_>>();
                    let tasks = paths.into_iter().map(|path| {
                        assert!(
                            journal_includes.contains(&path),
                            "detected file change for a file that is not included"
                        );
                        Task::perform(Journal::load(path), Message::Updated)
                    });
                    Task::batch(tasks)
                }
                _ => unreachable!(),
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.journal {
            Promise::Loading => text(format!("loading {}", self.path.display())).into(),
            Promise::Loaded(Err(hledger_journal::Error::Io(kind))) => text(kind.to_string()).into(),
            Promise::Loaded(Err(hledger_journal::Error::Glob(error))) => {
                text(error.to_string()).into()
            }
            Promise::Loaded(Err(hledger_journal::Error::Parse(_))) => text("parse error").into(),
            Promise::Loaded(Ok(_)) => {
                scrollable(List::new(&self.content, |_, tx| view_transaction(tx)))
                    .width(Length::Fill)
                    .into()
            }
        }
    }

    #[allow(clippy::unused_self)]
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(watcher::run).map(Message::Watcher)
    }
}

fn view_transaction(tx: &hledger_journal::Transaction) -> Element<Message> {
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
}
