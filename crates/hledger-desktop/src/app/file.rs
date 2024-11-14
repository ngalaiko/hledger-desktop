use std::collections::HashSet;

use iced::futures::channel::mpsc;
use iced::futures::SinkExt;
use iced::widget::text;
use iced::{Element, Subscription, Task};

use crate::journal::Journal;
use crate::promise::Promise;
use crate::{journal, watcher};

#[derive(Debug)]
pub struct File {
    pub path: std::path::PathBuf,
    watcher_input: Option<mpsc::Sender<watcher::Input>>,
    journal: Promise<Result<journal::Journal, journal::LoadError>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Watcher(watcher::Message),
    Loaded(Result<journal::Journal, journal::LoadError>),
    Updated(Result<journal::Journal, journal::LoadError>),
}

impl File {
    pub fn new(path: std::path::PathBuf) -> (Self, Task<Message>) {
        (
            Self {
                path: path.clone(),
                watcher_input: None,
                journal: Promise::Loading,
            },
            Task::perform(journal::Journal::load(path), Message::Loaded),
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
            Promise::Loading => text(format!("loading {}", self.path.display())),
            Promise::Loaded(Err(journal::LoadError::Io(kind))) => text(kind.to_string()),
            Promise::Loaded(Err(journal::LoadError::Glob(error))) => text(error.to_string()),
            Promise::Loaded(Err(journal::LoadError::Parse(_))) => text("parse error"),
            Promise::Loaded(Ok(journal::Journal {
                path, directives, ..
            })) => text(format!(
                "loaded {}, {} directives",
                path.display(),
                directives.len()
            )),
        }
        .into()
    }

    #[allow(clippy::unused_self)]
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(watcher::run).map(Message::Watcher)
    }
}
