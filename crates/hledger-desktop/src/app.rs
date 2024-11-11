mod file;

use iced::widget::{button, column, row, text};
use iced::{Element, Task};

use self::file::File;

#[derive(Debug, Default)]
pub struct HledgerDesktop {
    file: Option<File>,
}

#[derive(Debug, Clone)]
pub enum Message {
    File(file::Message),
    OpenFileDialog,
    FileSelected(Option<std::path::PathBuf>),
}

impl HledgerDesktop {
    #[allow(clippy::unused_self)]
    pub fn title(&self) -> String {
        String::from("hledger-deskop")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::File(message) => match &mut self.file {
                Some(file) => file.update(message).map(Message::File),
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
}

async fn select_file() -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .map(|file_handle| file_handle.path().to_path_buf())
}
