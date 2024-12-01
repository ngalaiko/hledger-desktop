mod register;

use std::collections::HashSet;

use hledger_journal::Journal;
use iced::futures::channel::mpsc;
use iced::futures::SinkExt;
use iced::widget::text;
use iced::{Element, Subscription, Task};

use crate::watcher;

use self::register::Register;

#[derive(Debug)]
pub struct File {
    pub path: std::path::PathBuf,
    watcher_input: Option<mpsc::Sender<watcher::Input>>,
    journal: Option<Result<hledger_journal::Journal, hledger_journal::Error>>,
    register: Register,
}

#[derive(Debug, Clone)]
pub enum Message {
    Register(register::Message),
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
                journal: None,
                register: Register::new(),
            },
            Task::perform(hledger_journal::Journal::load(path), Message::Loaded),
        )
    }

    #[allow(clippy::unused_self)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Register(message) => self.register.update(message).map(Message::Register),
            Message::Updated(result) => match (&mut self.journal, &result) {
                (Some(Ok(journal)), Ok(updated)) => {
                    journal.merge(updated);
                    self.register = Register::from_journal(journal);
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
                        self.register = Register::from_journal(journal);
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
                self.journal = Some(result);
                task
            }
            Message::Watcher(watcher::Message::Started(watcher_input)) => {
                assert!(self.watcher_input.is_none(), "watcher started twice");
                self.watcher_input.replace(watcher_input);
                Task::none()
            }
            Message::Watcher(watcher::Message::FileChange(paths)) => match &self.journal {
                Some(Ok(journal)) => {
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
            None => text(format!("loading {}", self.path.display())).into(),
            Some(Err(hledger_journal::Error::Io(kind))) => text(kind.to_string()).into(),
            Some(Err(hledger_journal::Error::Glob(error))) => text(error.to_string()).into(),
            Some(Err(hledger_journal::Error::Parse(_))) => text("parse error").into(),
            Some(Ok(_)) => self.register.view().map(Message::Register),
        }
    }

    #[allow(clippy::unused_self)]
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(watcher::run).map(Message::Watcher)
    }
}
