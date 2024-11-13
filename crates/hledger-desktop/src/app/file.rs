use std::collections::HashSet;

use iced::futures::channel::mpsc;
use iced::futures::SinkExt;
use iced::widget::text;
use iced::{Element, Task};

use crate::journal::Journal;
use crate::promise::Promise;
use crate::{journal, watcher};

#[derive(Debug)]
pub struct File {
    path: std::path::PathBuf,
    watcher_input: mpsc::Sender<watcher::Input>,
    journal: Promise<Result<journal::Journal, journal::LoadError>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<journal::Journal, journal::LoadError>),
    Updated(Result<journal::Journal, journal::LoadError>),
    FilesChanged(Vec<std::path::PathBuf>),
}

impl File {
    pub fn new(
        path: std::path::PathBuf,
        watcher_input: mpsc::Sender<watcher::Input>,
    ) -> (Self, Task<Message>) {
        (
            Self {
                path: path.clone(),
                watcher_input,
                journal: Promise::Loading,
            },
            Task::perform(journal::Journal::load(path), Message::Loaded),
        )
    }

    #[allow(clippy::unused_self)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Updated(result) => {
                if let Ok(journal) = result {
                    // TODO: handle journal update
                    dbg!(journal.path);
                }
                Task::none()
            }
            Message::Loaded(result) => {
                let task = if let Ok(journal) = &result {
                    let journal_includes = journal.includes();
                    let mut watcher_input = self.watcher_input.clone();
                    Task::future(async move {
                        watcher_input
                            .send(watcher::Input::Watch(journal_includes))
                            .await
                    })
                    .then(|_| Task::none())
                } else {
                    Task::none()
                };
                self.journal = Promise::Loaded(result);
                task
            }
            Message::FilesChanged(paths) => match &self.journal {
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
}
