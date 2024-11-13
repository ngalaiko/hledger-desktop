mod file;

use iced::futures::channel::mpsc;
use iced::widget::{button, column, row, text};
use iced::{Element, Subscription, Task};

use crate::watcher;

use self::file::File;

#[derive(Debug, Default)]
pub struct App {
    file: Option<File>,
    watcher_input: Option<mpsc::Sender<watcher::Input>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFileDialog,
    FileSelected(Option<std::path::PathBuf>),

    File(file::Message),
    Watcher(watcher::Message),
}

impl App {
    #[allow(clippy::unused_self)]
    pub fn title(&self) -> String {
        String::from("hledger-deskop")
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
                let watcher_input = self
                    .watcher_input
                    .clone()
                    .expect("watcher initialized before file is selected");
                if let Some(path) = selected_path {
                    let (file, task) = File::new(path, watcher_input);
                    self.file = Some(file);
                    task.map(Message::File)
                } else {
                    Task::none()
                }
            }
            Message::Watcher(event) => match event {
                watcher::Message::Started(watcher_input) => {
                    assert!(self.watcher_input.is_none(), "watcher started twice");
                    self.watcher_input.replace(watcher_input);
                    Task::none()
                }
                watcher::Message::FileChange(paths) => match &mut self.file {
                    None => unreachable!(),
                    Some(file) => file
                        .update(file::Message::FilesChanged(paths))
                        .map(Message::File),
                },
            },
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
    pub fn file_watcher(&self) -> Subscription<Message> {
        Subscription::run(watcher::run).map(Message::Watcher)
    }
}

async fn select_file() -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .map(|file_handle| file_handle.path().to_path_buf())
}
