use iced::widget::text;
use iced::{Element, Task};

use crate::journal;

#[derive(Debug)]
pub struct File {
    path: std::path::PathBuf,
    state: State,
}

#[derive(Debug)]
enum State {
    Loading,
    Loaded(Result<journal::Journal, journal::LoadError>),
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
                state: State::Loading,
            },
            Task::perform(journal::Journal::load(path), Message::Loaded),
        )
    }

    #[allow(clippy::unused_self)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded(result) => self.state = State::Loaded(result),
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        match &self.state {
            State::Loading => text(format!("loading {}", self.path.display())),
            State::Loaded(Err(journal::LoadError::Io(kind))) => text(kind.to_string()),
            State::Loaded(Err(journal::LoadError::Glob(error))) => text(error.to_string()),
            State::Loaded(Err(journal::LoadError::Parse(_))) => text("parse error"),
            State::Loaded(Ok(journal::Journal {
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
