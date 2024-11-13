use iced::widget::text;
use iced::{Element, Task};

use crate::journal;
use crate::promise::Promise;

#[derive(Debug)]
pub struct File {
    path: std::path::PathBuf,
    journal: Promise<Result<journal::Journal, journal::LoadError>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<journal::Journal, journal::LoadError>),
}

impl File {
    pub fn new(path: std::path::PathBuf) -> (Self, Task<Message>) {
        (
            Self {
                path: path.clone(),
                journal: Promise::Loading,
            },
            Task::perform(journal::Journal::load(path), Message::Loaded),
        )
    }

    #[allow(clippy::unused_self)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded(result) => self.journal = Promise::Loaded(result),
        }
        Task::none()
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
