use iced::widget::text;
use iced::{Center, Element, Fill, Task as Command};

#[allow(clippy::missing_errors_doc)]
pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(
        HledgerDesktop::title,
        HledgerDesktop::update,
        HledgerDesktop::view,
    )
    .window_size((500.0, 800.0))
    .run()
}

#[derive(Debug, Default)]
struct HledgerDesktop {}

#[derive(Debug, Clone, Copy)]
enum Message {}

impl HledgerDesktop {
    #[allow(clippy::unused_self)]
    fn title(&self) -> String {
        String::from("hledger-deskop")
    }

    #[allow(clippy::unused_self)]
    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    #[allow(clippy::unused_self)]
    fn view(&self) -> Element<Message> {
        text("hello world")
            .width(Fill)
            .size(100)
            .align_x(Center)
            .align_y(Center)
            .into()
    }
}
