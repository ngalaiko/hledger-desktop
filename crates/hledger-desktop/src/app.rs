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
    File(file::Message),
    OpenFileDialog,
    FileSelected(Option<std::path::PathBuf>),
    Watcher(watcher::Event),
}

impl App {
    #[allow(clippy::unused_self)]
    pub fn title(&self) -> String {
        String::from("hledger-deskop")
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::File(message) => match &mut self.file {
                Some(file) => {
                    assert!(
                        self.watcher_input.is_some(),
                        "watching files before watcher is running"
                    );
                    if let (file::Message::Loaded(Ok(journal)), Some(input)) =
                        (&message, &mut self.watcher_input)
                    {
                        for file in journal.includes() {
                            input.try_send(watcher::Input::Watch(file)).unwrap();
                        }
                    }
                    file.update(message).map(Message::File)
                }
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
            Message::Watcher(event) => {
                match event {
                    watcher::Event::Started(watcher_input) => {
                        assert!(self.watcher_input.is_none(), "watcher started twice");
                        self.watcher_input.replace(watcher_input);
                    }
                    watcher::Event::ChangeEvent(paths) => {
                        dbg!(&paths);
                    }
                }
                Task::none()
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
