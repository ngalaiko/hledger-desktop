mod file;

use iced::widget::{button, column, row, text};
use iced::{Element, Subscription, Task};

use self::file::File;

#[derive(Debug, Default)]
pub struct App {
    file: Option<File>,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFileDialog,
    FileSelected(Option<std::path::PathBuf>),

    File(file::Message),
}

impl App {
    pub fn title(&self) -> String {
        if let Some(file_name) = self
            .file
            .as_ref()
            .and_then(|file| file.path.file_name())
            .and_then(|file_name| file_name.to_str())
        {
            file_name.to_string()
        } else {
            String::from("hledger-deskop")
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::File(file_message) => match &mut self.file {
                Some(file) => file.update(file_message).map(Message::File),
                None => unreachable!(),
            },
            Message::OpenFileDialog => match self.file {
                Some(_) => Task::none(),
                None => Task::perform(select_file(), Message::FileSelected),
            },
            Message::FileSelected(selected_path) => {
                if let Some(path) = selected_path {
                    let (file, task) = File::new(path);
                    self.file = Some(file);
                    task.map(Message::File)
                } else {
                    Task::none()
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let open_file_button = button(text("Open File")).on_press(Message::OpenFileDialog);
        let controls = column![row![open_file_button]];

        if let Some(file) = self.file.as_ref() {
            file.view().map(Message::File)
        } else {
            controls.into()
        }
    }

    #[allow(clippy::unused_self)]
    pub fn subscription(&self) -> Subscription<Message> {
        if let Some(file) = &self.file {
            file.subscription().map(Message::File)
        } else {
            Subscription::none()
        }
    }
}

async fn select_file() -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .map(|file_handle| file_handle.path().to_path_buf())
}
