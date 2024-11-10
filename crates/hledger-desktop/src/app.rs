use iced::widget::{button, column, container, row, text, tooltip};
use iced::{Center, Element, Font, Task};

use crate::journal;

#[derive(Debug, Default)]
pub struct HledgerDesktop {
    file: Option<File>,
}

#[derive(Debug)]
pub enum File {
    Loading(std::path::PathBuf),
    Error(journal::LoadError),
    Loaded(journal::Journal),
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFileDialog,
    FileSelected(Option<std::path::PathBuf>),
    FileOpened(Result<journal::Journal, journal::LoadError>),
}

impl HledgerDesktop {
    #[allow(clippy::unused_self)]
    pub fn title(&self) -> String {
        String::from("hledger-deskop")
    }

    #[allow(clippy::unused_self)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFileDialog => match self.file {
                Some(_) => Task::none(),
                None => Task::perform(select_file(), Message::FileSelected),
            },
            Message::FileSelected(selected_path) => {
                if let Some(path) = selected_path {
                    self.file = Some(File::Loading(path.clone()));
                    Task::perform(journal::Journal::load(path), Message::FileOpened)
                } else {
                    Task::none()
                }
            }
            Message::FileOpened(result) => {
                match result {
                    Ok(journal) => self.file = Some(File::Loaded(journal)),
                    Err(error) => self.file = Some(File::Error(error)),
                }
                Task::none()
            }
        }
    }

    #[allow(clippy::unused_self)]
    pub fn view(&self) -> Element<Message> {
        let controls = row![action(
            open_icon(),
            "Open file",
            self.file.is_none().then_some(Message::OpenFileDialog)
        )]
        .spacing(10)
        .align_y(Center);

        let content = match &self.file {
            None => text("empty"),
            Some(File::Loading(path)) => text(format!("loading {}", path.display())),
            Some(File::Error(journal::LoadError::Io(kind))) => text(kind.to_string()),
            Some(File::Error(journal::LoadError::Glob(error))) => text(error.to_string()),
            Some(File::Error(journal::LoadError::Parse(_))) => text("parse error"),
            Some(File::Loaded(journal::Journal {
                path, directives, ..
            })) => text(format!(
                "loaded {}, {} directives",
                path.display(),
                directives.len()
            )),
        };

        column![controls, content].into()
    }
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}

fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(container(content).center_x(30));

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(container::rounded_box)
        .into()
    } else {
        action.style(button::secondary).into()
    }
}

async fn select_file() -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .map(|file_handle| file_handle.path().to_path_buf())
}
